import subprocess
import sys

if __name__ == "__main__":
    seed = int(sys.argv[1])
    file = f"{seed:04}"

    subprocess.run("cargo build --features local --release", shell=True)
    subprocess.run(
        "./tools/target/release/tester ./target/release/ahc025"
        + f"< tools/in/{file}.txt > tools/out/{file}.txt",
        shell=True,
    )
    subprocess.run(f"pbcopy < tools/out/{file}.txt", shell=True)
