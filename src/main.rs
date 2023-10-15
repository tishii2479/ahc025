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

    // 集合の重さの差が改善しなければ不採用
    match balancer.get_result(
        &groups[rank[lighter_g_idx]],
        &groups[rank[heavier_g_idx]],
        interactor,
    ) {
        BalanceResult::Right | BalanceResult::Equal => return false,
        _ => {}
    }

    if !update_rank(
        rank,
        &groups,
        true,
        lighter_g_idx,
        heavier_g_idx,
        input,
        interactor,
        balancer,
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
        balancer,
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
    match balancer.get_result(&vec![item_idx_a], &vec![item_idx_b], interactor) {
        BalanceResult::Left | BalanceResult::Equal => {}
        _ => return false,
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
            return false;
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
                balancer,
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
                balancer,
            ) {
                return false;
            }
            return true;
        }
    }
}

///
/// 1. A < Bとする
/// 2. A、Bからランダムにアイテムa_1, b_1を選択する
/// 3. a_1 > b_1の時
///     1. a_1 < b_1 + b_2となるようにアイテムb_2をBから選択する
/// 4. a_1 < b_1の時
///     1. a_1 + a_2 < b_1となるようなa_2があればAから選択する
///
fn action_swap2(
    heavier_g_idx: usize,
    lighter_g_idx: usize,
    groups: &mut Vec<Vec<usize>>,
    rank: &mut Vec<usize>,
    input: &Input,
    balancer: &mut Balancer,
    interactor: &mut Interactor,
) -> bool {
    let a1 = rnd::gen_range(0, groups[rank[lighter_g_idx]].len());
    let b1 = rnd::gen_range(0, groups[rank[heavier_g_idx]].len());
    let mut item_indices_a = vec![groups[rank[lighter_g_idx]][a1]];
    let mut item_indices_b = vec![groups[rank[heavier_g_idx]][b1]];
    let mut item_indices_in_a = vec![a1];
    let mut item_indices_in_b = vec![b1];

    // 入れ替えようとしているアイテムの大小関係が集合の大小関係と一致しなければ不採用
    match balancer.get_result(&item_indices_a, &item_indices_b, interactor) {
        BalanceResult::Right => {
            for _ in 0..3 {
                let b2 = rnd::gen_range(0, groups[rank[heavier_g_idx]].len());
                if item_indices_in_b.contains(&b2) {
                    continue;
                }
                item_indices_b.push(groups[rank[heavier_g_idx]][b2]);
                item_indices_in_b.push(b2);
                match balancer.get_result(&item_indices_a, &item_indices_b, interactor) {
                    BalanceResult::Left | BalanceResult::Equal => break,
                    _ => {
                        item_indices_b.pop();
                        item_indices_in_b.pop();
                    }
                }
            }
        }
        BalanceResult::Left => {
            for _ in 0..3 {
                let a2 = rnd::gen_range(0, groups[rank[lighter_g_idx]].len());
                if item_indices_in_a.contains(&a2) {
                    continue;
                }
                item_indices_a.push(groups[rank[lighter_g_idx]][a2]);
                item_indices_in_a.push(a2);
                match balancer.get_result(&item_indices_a, &item_indices_b, interactor) {
                    BalanceResult::Left | BalanceResult::Equal => break,
                    _ => {
                        item_indices_a.pop();
                        item_indices_in_a.pop();
                    }
                }
            }
        }
        BalanceResult::Equal => {}
        BalanceResult::Unknown => return false,
    }
    match balancer.get_result(&item_indices_a, &item_indices_b, interactor) {
        BalanceResult::Left | BalanceResult::Equal => {}
        _ => return false,
    }

    for item_idx_a in item_indices_a.iter() {
        let i = groups[rank[lighter_g_idx]]
            .iter()
            .position(|e| e == item_idx_a)
            .unwrap();
        groups[rank[lighter_g_idx]].remove(i);
    }
    for item_idx_b in item_indices_b.iter() {
        let i = groups[rank[heavier_g_idx]]
            .iter()
            .position(|e| e == item_idx_b)
            .unwrap();
        groups[rank[heavier_g_idx]].remove(i);
    }

    match balancer.get_result(
        &groups[rank[lighter_g_idx]],
        &groups[rank[heavier_g_idx]],
        interactor,
    ) {
        // 集合の重さの差が悪化したら不採用
        BalanceResult::Right | BalanceResult::Unknown => {
            for item_idx_a in item_indices_a.iter() {
                groups[rank[lighter_g_idx]].push(*item_idx_a);
            }
            for item_idx_b in item_indices_b.iter() {
                groups[rank[heavier_g_idx]].push(*item_idx_b);
            }
            return false;
        }
        _ => {
            for item_idx_a in item_indices_a.iter() {
                groups[rank[heavier_g_idx]].push(*item_idx_a);
            }
            if !update_rank(
                rank,
                &groups,
                true,
                lighter_g_idx,
                heavier_g_idx,
                input,
                interactor,
                balancer,
            ) {
                return false;
            }
            for item_idx_b in item_indices_b.iter() {
                groups[rank[lighter_g_idx]].push(*item_idx_b);
            }
            if !update_rank(
                rank,
                &groups,
                false,
                lighter_g_idx,
                heavier_g_idx,
                input,
                interactor,
                balancer,
            ) {
                return false;
            }
            dbg!(&item_indices_a, &item_indices_b);
            return true;
        }
    }
}

fn select_g_idx_pair(groups: &Vec<Vec<usize>>, rank: &Vec<usize>, input: &Input) -> (usize, usize) {
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
    (lighter_g_idx, heavier_g_idx)
}

fn solve(input: &Input, interactor: &mut Interactor) {
    let mut balancer = Balancer::new(input);

    // ランダムにグループに割り振る
    let mut groups = vec![vec![]; input.d];
    for i in 0..input.n {
        groups[i % input.d].push(i);
    }

    // ソートして順位をつける
    let mut rank = sort_groups(&groups, input, interactor, &mut balancer);
    eprintln!("after_sort: {} / {}", interactor.query_count, input.q);

    let mut trial_count = 0;
    let mut move_adopted_count = 0;
    let mut swap_adopted_count = 0;
    let mut swap2_adopted_count = 0;

    while interactor.query_count < input.q && time::elapsed_seconds() < TIME_LIMIT - 0.2 {
        // TODO: ロールバックの高速化
        let copied_groups = groups.clone();
        let (lighter_g_idx, heavier_g_idx) = select_g_idx_pair(&groups, &rank, input);

        let p = rnd::nextf();
        let action = if p < 0.5 {
            action_move
        } else if p < 0.9 {
            action_swap
        } else {
            action_swap2
        };

        trial_count += 1;
        if action(
            heavier_g_idx,
            lighter_g_idx,
            &mut groups,
            &mut rank,
            input,
            &mut balancer,
            interactor,
        ) {
            if action == action_move {
                move_adopted_count += 1;
                eprintln!("[{} / {}] adopt move", interactor.query_count, input.q);
            } else if action == action_swap {
                swap_adopted_count += 1;
                eprintln!("[{} / {}] adopt swap", interactor.query_count, input.q);
            } else if action == action_swap2 {
                swap2_adopted_count += 1;
                eprintln!("[{} / {}] adopt swap2", interactor.query_count, input.q);
            }
        } else {
            groups = copied_groups;
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
    eprintln!("swap2_adopted_count: {swap2_adopted_count}");

    let d = groups_to_output_d(&groups, input);
    interactor.output_d(&d, false);
}

fn main() {
    time::start_clock();

    let mut interactor = Interactor::new();
    let input = interactor.read_input();

    solve(&input, &mut interactor);
}
