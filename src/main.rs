mod def;
mod interactor;
mod util;

use crate::def::*;
use crate::interactor::*;
use crate::util::*;

fn select_lighter_item(group: &Vec<usize>, balancer: &mut Balancer) -> usize {
    let item_idx_in_group = rnd::gen_range(0, group.len());
    let item_idx = group[item_idx_in_group];
    let item_idx = balancer.find_lighter_in_group(item_idx, &group);
    item_idx
}

fn action_move(
    heavier_g_idx: usize,
    lighter_g_idx: usize,
    groups: &mut Vec<Vec<usize>>,
    rank: &mut Vec<usize>,
    input: &Input,
    balancer: &mut Balancer,
    interactor: &mut Interactor,
) -> bool {
    let item_idx = select_lighter_item(&groups[rank[heavier_g_idx]], balancer);
    let i = groups[rank[heavier_g_idx]]
        .iter()
        .position(|x| *x == item_idx)
        .unwrap();
    groups[rank[heavier_g_idx]].swap_remove(i);

    // 集合の重さの差が改善しなければ不採用
    match balancer.get_result(
        &groups[rank[lighter_g_idx]],
        &groups[rank[heavier_g_idx]],
        interactor,
    ) {
        BalanceResult::Right | BalanceResult::Equal => {
            groups[rank[heavier_g_idx]].push(item_idx);
            return false;
        }
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
        // 計測できなかった場合はとりあえず元に戻す
        groups[rank[heavier_g_idx]].push(item_idx);
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
        groups[rank[heavier_g_idx]].push(item_idx);
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
                groups[rank[lighter_g_idx]].push(item_idx_b);
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
    trial_count: usize,
    heavier_g_idx: usize,
    lighter_g_idx: usize,
    groups: &mut Vec<Vec<usize>>,
    rank: &mut Vec<usize>,
    input: &Input,
    balancer: &mut Balancer,
    interactor: &mut Interactor,
) -> bool {
    // NOTE: trial_count = 0にすればaction_swapと一緒の挙動？
    let a1 = rnd::gen_range(0, groups[rank[lighter_g_idx]].len());
    let b1 = rnd::gen_range(0, groups[rank[heavier_g_idx]].len());
    let mut item_indices_a = vec![groups[rank[lighter_g_idx]][a1]];
    let mut item_indices_b = vec![groups[rank[heavier_g_idx]][b1]];

    // 入れ替えようとしているアイテムの大小関係が集合の大小関係と一致しなければ不採用
    match balancer.get_result(&item_indices_a, &item_indices_b, interactor) {
        // 重い方に大小関係が入れ替わるものがあれば足す
        BalanceResult::Right => {
            for _ in 0..trial_count {
                let b2 = select_lighter_item(&groups[rank[heavier_g_idx]], balancer);
                if item_indices_b.contains(&b2) {
                    continue;
                }
                item_indices_b.push(b2);
                match balancer.get_result(&item_indices_a, &item_indices_b, interactor) {
                    BalanceResult::Left | BalanceResult::Equal => break,
                    _ => {
                        item_indices_b.pop();
                    }
                }
            }
        }
        BalanceResult::Left => {
            // 軽い方に足せるものがあれば足す
            for _ in 0..trial_count {
                let a2 = select_lighter_item(&groups[rank[lighter_g_idx]], balancer);
                if item_indices_a.contains(&a2) {
                    continue;
                }
                item_indices_a.push(a2);
                match balancer.get_result(&item_indices_a, &item_indices_b, interactor) {
                    BalanceResult::Left | BalanceResult::Equal => continue,
                    _ => {
                        item_indices_a.pop();
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
                for item_idx_b in item_indices_b.iter() {
                    groups[rank[lighter_g_idx]].push(*item_idx_b);
                }
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
            if item_indices_a.len() > 1 || item_indices_b.len() > 1 {
                eprintln!("swap2: {:?} {:?}", item_indices_a, item_indices_b);
            }
            return true;
        }
    }
}

fn select_g_idx_pair(input: &Input) -> (usize, usize) {
    const P: f64 = 0.3;
    let par = 1 + (time::elapsed_seconds() * 5.0 / (TIME_LIMIT - 0.1)).round() as usize;
    let mut lighter_g_idx = 0;
    let mut heavier_g_idx = input.d - 1;
    for i in 0..par.min(input.d / 2) {
        if rnd::nextf() < P {
            lighter_g_idx = i;
            break;
        }
    }
    for i in ((input.d - input.d.min(par)).max(input.d / 2)..input.d).rev() {
        if rnd::nextf() < P {
            heavier_g_idx = i;
            break;
        }
    }
    (lighter_g_idx, heavier_g_idx)
}

fn solve(input: &Input, interactor: &mut Interactor) {
    let mut balancer = Balancer::new();

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

    while interactor.query_count < input.q && time::elapsed_seconds() < TIME_LIMIT - 0.1 {
        let (lighter_g_idx, heavier_g_idx) = select_g_idx_pair(input);
        trial_count += 1;

        let p = rnd::nextf();
        if p < 0.5 {
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
                eprintln!("[{} / {}] adopt move", interactor.query_count, input.q);
            }
        } else if p < 0.9 {
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
                eprintln!("[{} / {}] adopt swap", interactor.query_count, input.q);
            }
        } else {
            const TRIAL_COUNT: usize = 3;
            if action_swap2(
                TRIAL_COUNT,
                heavier_g_idx,
                lighter_g_idx,
                &mut groups,
                &mut rank,
                input,
                &mut balancer,
                interactor,
            ) {
                swap2_adopted_count += 1;
                eprintln!("[{} / {}] adopt swap2", interactor.query_count, input.q);
            }
        };

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
