import argparse
import datetime
import json
import logging
import subprocess
from dataclasses import dataclass
from logging import FileHandler, StreamHandler, getLogger
from typing import List, Optional

import pandas as pd
from joblib import Parallel, delayed

TIME_LIMIT = 61.0


def setup_logger() -> logging.Logger:
    logger = getLogger(__name__)
    logger.setLevel(logging.INFO)

    stream_handler = StreamHandler()
    stream_handler.setLevel(logging.DEBUG)
    file_handler = FileHandler("log/a.log", "a")
    file_handler.setLevel(logging.DEBUG)

    logger.addHandler(stream_handler)
    logger.addHandler(file_handler)

    logger.debug("Hello World!")
    return logger


logger = setup_logger()


@dataclass
class Param:
    mark_count: int
    measure_count: int

    def to_str(self) -> str:
        return f"{self.mark_count} {self.measure_count}"

    def to_dict(self) -> dict:
        return {"mark_count": self.mark_count, "measure_count": self.measure_count}


@dataclass
class Result:
    input_file: str
    solver_version: str
    score: int
    duration: float

    def __init__(self, result_json: dict, input_file: str, solver_version: str):
        self.input_file = input_file
        self.solver_version = solver_version
        self.score = result_json["score"]
        self.duration = result_json["duration"]


@dataclass
class Input:
    n: int
    d: int
    q: int

    def __init__(self, in_file: str):
        with open(in_file, "r") as f:
            n, d, q = map(int, f.readline().split())
            _ = list(map(int, f.readline().split()))
        self.n = n
        self.d = d
        self.q = q


def run_case(
    input_file: str, output_file: str, solver_version: str, solver_cmd: str
) -> Result:
    while True:
        cmd = f"{solver_cmd} < {input_file} > {output_file}"
        proc = subprocess.run(
            cmd, shell=True, stderr=subprocess.PIPE, timeout=TIME_LIMIT
        )
        stderr = proc.stderr.decode("utf-8")
        result_str = stderr[stderr.find("result:") + len("result:") :]
        try:
            result_json = json.loads(result_str)
            break
        except json.JSONDecodeError as e:
            logger.error(e)
            logger.error(
                f"failed to parse result_str: {result_str}, input_file: {input_file}"
            )

    result = Result(result_json, input_file, solver_version)
    return result


def run(
    data_dir: str,
    solver_path: str,
    solver_version: str,
    case_num: int,
    database_csv: str,
    args: str = "",
) -> pd.DataFrame:
    tester_path = "./tools/target/release/tester"
    solver_cmd = f"{tester_path} {solver_path} {args}"

    start_seed = 0
    cases = [
        (f"{data_dir}/in/{seed:04}.txt", f"{data_dir}/out/{seed:04}.txt")
        for seed in range(start_seed, start_seed + case_num)
    ]
    inputs = list(map(lambda x: Input(x[0]), cases))
    results = Parallel(n_jobs=-1, verbose=10)(
        delayed(run_case)(input_file, output_file, solver_version, solver_cmd)
        for input_file, output_file in cases
    )
    df = pd.DataFrame(
        list(map(lambda x: vars(x[0]) | vars(x[1]), zip(results, inputs)))
    )

    try:
        database_df = pd.read_csv(database_csv)
        database_df = pd.concat([database_df, df], axis=0, ignore_index=True)
        database_df.to_csv(database_csv, index=False)
    except (FileNotFoundError, pd.errors.EmptyDataError):
        logger.info(f"database_csv: {database_csv} not found, create new database_csv")
        df.to_csv(database_csv, index=False)

    return df


def evaluate_absolute_score(
    solver_version: str,
    database_csv: str,
    columns: Optional[List[str]] = None,
    eval_items: List[str] = ["score"],
) -> None:
    logger.info(f"Evaluate {solver_version}")
    database_df = pd.read_csv(database_csv)
    score_df = database_df[database_df.solver_version == solver_version].reset_index(
        drop=True
    )

    logger.info(f"Raw score mean: {score_df.score.mean()}")
    logger.info("Top 10 improvements:")
    logger.info(score_df.sort_values(by="score")[:10])
    logger.info("Top 10 aggravations:")
    logger.info(score_df.sort_values(by="score", ascending=False)[:10])
    logger.info(
        f"Longest duration: {score_df.sort_values(by='duration').duration.iloc[-1]}"
    )
    logger.info(score_df.score.describe())

    if columns is not None:
        assert 1 <= len(columns) <= 2
        if len(columns) == 1:
            logger.info(score_df.groupby(columns[0])["score"].mean())
        elif len(columns) == 2:
            logger.info(
                score_df[eval_items + columns].pivot_table(
                    index=columns[0], columns=columns[1]
                )
            )


def evaluate_relative_score(
    solver_version: str,
    benchmark_solver_version: str,
    database_csv: str,
    columns: Optional[List[str]] = None,
    eval_items: List[str] = ["score"],
) -> None:
    logger.info(f"Comparing {solver_version} -> {benchmark_solver_version}")
    database_df = pd.read_csv(database_csv)
    score_df = database_df[database_df.solver_version == solver_version].reset_index(
        drop=True
    )
    benchmark_df = database_df[
        database_df.solver_version == benchmark_solver_version
    ].reset_index(drop=True)

    score_df.loc[:, "relative_score"] = score_df.score / benchmark_df.score

    logger.info(f"Raw score mean: {score_df.score.mean()}")
    logger.info(f"Relative score mean: {score_df['relative_score'].mean()}")
    logger.info("Top 10 improvements:")
    logger.info(score_df.sort_values(by="relative_score")[:10])
    logger.info("Top 10 aggravations:")
    logger.info(score_df.sort_values(by="relative_score", ascending=False)[:10])
    logger.info(
        f"Longest duration: {score_df.sort_values(by='duration').duration.iloc[-1]}"
    )
    logger.info(score_df.score.describe())
    logger.info(score_df.relative_score.describe())
    logger.info(f"improve case count:   {(score_df.relative_score < 1.0).sum()}")
    logger.info(f"aggravate case count: {(score_df.relative_score > 1.0).sum()}")

    if columns is not None:
        assert 1 <= len(columns) <= 2
        if len(columns) == 1:
            # score_df["group"] = score_df["q"] // 300 * 300
            score_df["group"] = score_df["d"] // 3 * 3
            score_df["compare_result_win"] = score_df.relative_score < 1.0
            score_df["compare_result_lose"] = score_df.relative_score > 1.0
            logger.info(
                pd.merge(
                    score_df.groupby("group")["compare_result_win"].mean(),
                    score_df.groupby("group")["compare_result_lose"].mean(),
                    on="group",
                )
            )
        elif len(columns) == 2:
            logger.info(
                score_df[eval_items + columns].pivot_table(
                    index=columns[0], columns=columns[1]
                )
            )


def list_solvers(database_csv: str) -> None:
    database_df = pd.read_csv(database_csv)
    logger.info(
        database_df.groupby("solver_version")["score"].agg("mean").sort_values()[:50]
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("-d", "--data-dir", type=str, default="tools")
    parser.add_argument("-e", "--eval", action="store_true")
    parser.add_argument("-l", "--list-solver", action="store_true")
    parser.add_argument("-n", "--case_num", type=int, default=100)
    parser.add_argument(
        "-s", "--solver-path", type=str, default="./target/release/ahc025"
    )
    parser.add_argument(
        "-a",
        "--solver-version",
        type=str,
        default=f"solver-{datetime.datetime.now().strftime('%Y-%m-%d_%H:%M:%S')}",
    )
    parser.add_argument("-b", "--benchmark-solver-version", type=str, default=None)
    parser.add_argument("--database-csv", type=str, default="log/database.csv")
    args = parser.parse_args()

    if args.list_solver:
        list_solvers(args.database_csv)
    elif args.eval:
        if args.benchmark_solver_version is None:
            evaluate_absolute_score(args.solver_version, args.database_csv)
        else:
            evaluate_relative_score(
                args.solver_version,
                args.benchmark_solver_version,
                args.database_csv,
                columns=["q"],
            )
    else:
        subprocess.run("cargo build --features local --release", shell=True)
        subprocess.run(
            f"python3 expander.py > log/backup/{args.solver_version}.rs", shell=True
        )
        run(
            args.data_dir,
            args.solver_path,
            args.solver_version,
            args.case_num,
            args.database_csv,
        )
        if args.benchmark_solver_version is None:
            evaluate_absolute_score(args.solver_version, args.database_csv)
        else:
            evaluate_relative_score(
                args.solver_version,
                args.benchmark_solver_version,
                args.database_csv,
                columns=["q"],
            )
