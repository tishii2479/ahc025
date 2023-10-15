mod def;
mod interactor;
mod util;

use crate::def::*;
use crate::interactor::*;
use crate::util::*;

fn action_move(
    heavier_g_idx: usize,
    lighter_g_idx: usize,
    groups: &mut Vec<Vec<usize>>,
    rank: &mut Vec<usize>,
    input: &Input,
    balancer: &mut Balancer,
    interactor: &mut Interactor,
) -> bool {
    let item_idx_in_group = rnd::gen_range(0, groups[rank[heavier_g_idx]].len());
    let item_idx = groups[rank[heavier_g_idx]][item_idx_in_group];

    groups[rank[heavier_g_idx]].swap_remove(item_idx_in_group);

    // 集合の重さの差が悪化したら不採用
    if balancer.get_result(
        &groups[rank[lighter_g_idx]],
        &groups[rank[heavier_g_idx]],
        interactor,
    ) == BalanceResult::Right
    {
        return false;
    }

    if !update_rank(
        rank,
        &groups,
        true,
        lighter_g_idx,
        heavier_g_idx,
        input,
        interactor,
    ) {
        return false;
    }
    groups[rank[lighter_g_idx]].push(item_idx);
    if !update_rank(
        rank,
        &groups,
        false,
        lighter_g_idx,
        heavier_g_idx,
        input,
        interactor,
    ) {
        return false;
    }
    true
}

fn action_swap(
    heavier_g_idx: usize,
    lighter_g_idx: usize,
    groups: &mut Vec<Vec<usize>>,
    rank: &mut Vec<usize>,
    input: &Input,
    balancer: &mut Balancer,
    interactor: &mut Interactor,
) -> bool {
    let item_idx_in_group_a = rnd::gen_range(0, groups[rank[lighter_g_idx]].len());
    let item_idx_in_group_b = rnd::gen_range(0, groups[rank[heavier_g_idx]].len());
    let item_idx_a = groups[rank[lighter_g_idx]][item_idx_in_group_a];
    let item_idx_b = groups[rank[heavier_g_idx]][item_idx_in_group_b];

    // 入れ替えようとしているアイテムの大小関係が集合の大小関係と一致しなければ不採用
    if balancer.get_result(&vec![item_idx_a], &vec![item_idx_b], interactor) != BalanceResult::Left
    {
        return false;
    }

    groups[rank[lighter_g_idx]].swap_remove(item_idx_in_group_a);
    groups[rank[heavier_g_idx]].swap_remove(item_idx_in_group_b);
    match balancer.get_result(
        &groups[rank[lighter_g_idx]],
        &groups[rank[heavier_g_idx]],
        interactor,
    ) {
        // 集合の重さの差が悪化したら不採用
        BalanceResult::Right | BalanceResult::Unknown => {
            groups[rank[lighter_g_idx]].push(item_idx_a);
            groups[rank[heavier_g_idx]].push(item_idx_b);
        }
        _ => {
            groups[rank[heavier_g_idx]].push(item_idx_a);
            if !update_rank(
                rank,
                &groups,
                true,
                lighter_g_idx,
                heavier_g_idx,
                input,
                interactor,
            ) {
                return false;
            }
            groups[rank[lighter_g_idx]].push(item_idx_b);
            if !update_rank(
                rank,
                &groups,
                false,
                lighter_g_idx,
                heavier_g_idx,
                input,
                interactor,
            ) {
                return false;
            }
        }
    }
    true
}

fn solve(input: &Input, interactor: &mut Interactor) {
    let mut balancer = Balancer::new(input);

    // ランダムにグループに割り振る
    let mut groups = vec![vec![]; input.d];
    for i in 0..input.n {
        groups[i % input.d].push(i);
    }

    // ソートして順位をつける
    let mut rank = sort_groups(&groups, input, interactor);
    eprintln!("after_sort: {} / {}", interactor.query_count, input.q);

    let mut trial_count = 0;
    let mut move_adopted_count = 0;
    let mut swap_adopted_count = 0;

    // 一番重いグループから軽いグループに移す
    while interactor.query_count < input.q && time::elapsed_seconds() < TIME_LIMIT - 0.2 {
        // TODO: ロールバックの高速化
        let copied_groups = groups.clone();

        let mut heavier_g_idx;
        let mut lighter_g_idx;
        loop {
            let par = 1 + (time::elapsed_seconds() * 5.0 / (TIME_LIMIT - 0.1)).round() as usize;
            lighter_g_idx = rnd::gen_range(0, par.min(input.d / 2));
            heavier_g_idx = rnd::gen_range((input.d - input.d.min(par)).max(input.d / 2), input.d);

            if groups[rank[heavier_g_idx]].len() > 1 {
                break;
            }
        }

        trial_count += 1;
        if rnd::nextf() < 0.5 {
            if action_move(
                heavier_g_idx,
                lighter_g_idx,
                &mut groups,
                &mut rank,
                input,
                &mut balancer,
                interactor,
            ) {
                move_adopted_count += 1;
                eprintln!("[{} / {}] adopt_move", interactor.query_count, input.q);
            } else {
                groups = copied_groups;
            }
        } else {
            if action_swap(
                heavier_g_idx,
                lighter_g_idx,
                &mut groups,
                &mut rank,
                input,
                &mut balancer,
                interactor,
            ) {
                swap_adopted_count += 1;
                eprintln!("[{} / {}] adopt_swap", interactor.query_count, input.q);
            } else {
                groups = copied_groups;
            }
        }

        let d = groups_to_output_d(&groups, input);
        interactor.output_d(&d, true);
    }

    // 必要ないクエリを消化する
    while interactor.query_count < input.q {
        interactor.output_query(&vec![0], &vec![1]);
    }

    eprintln!("trial_count:         {trial_count}");
    eprintln!("move_adopted_count:  {move_adopted_count}");
    eprintln!("swap_adopted_count:  {swap_adopted_count}");

    let d = groups_to_output_d(&groups, input);
    interactor.output_d(&d, false);
}

fn main() {
    time::start_clock();

    let mut interactor = Interactor::new();
    let input = interactor.read_input();

    solve(&input, &mut interactor);
}
