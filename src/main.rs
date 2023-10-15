mod def;
mod interactor;
mod util;

use crate::def::*;
use crate::interactor::*;
use crate::util::*;

fn groups_to_output_d(groups: &Vec<Vec<usize>>, input: &Input) -> Vec<usize> {
    let mut d = vec![0; input.n];
    for (g_idx, group) in groups.iter().enumerate() {
        for e in group {
            d[*e] = g_idx;
        }
    }
    d
}

fn sort_groups(groups: &Vec<Vec<usize>>, input: &Input, interactor: &mut Interactor) -> Vec<usize> {
    fn q_sort(
        targets: Vec<usize>,
        groups: &Vec<Vec<usize>>,
        input: &Input,
        interactor: &mut Interactor,
    ) -> Vec<usize> {
        if targets.len() <= 1 {
            return targets;
        }
        let pivot_g_idx = targets[rnd::gen_range(0, targets.len())];
        let mut left_targets = vec![pivot_g_idx];
        let mut right_targets = vec![];
        for g_idx in targets {
            if g_idx == pivot_g_idx {
                continue;
            }
            match interactor.output_query(&groups[pivot_g_idx], &groups[g_idx]) {
                BalanceResult::Left => right_targets.push(g_idx), // <
                BalanceResult::Right => left_targets.push(g_idx), // >
                BalanceResult::Equal => right_targets.push(g_idx), // =
                BalanceResult::Unknown => break,
            }
        }
        [
            q_sort(left_targets, groups, input, interactor),
            q_sort(right_targets, groups, input, interactor),
        ]
        .concat()
    }

    q_sort((0..groups.len()).collect(), groups, input, interactor)
}

fn update_rank(
    rank: &mut Vec<usize>,
    groups: &Vec<Vec<usize>>,
    from_up: bool,
    heaviest_g_idx: usize,
    input: &Input,
    interactor: &mut Interactor,
) -> bool {
    if input.d < 10 {
        update_rank_bubble(rank, groups, from_up, heaviest_g_idx, interactor)
    } else {
        update_rank_binary_search(rank, groups, from_up, heaviest_g_idx, interactor)
    }
}

fn update_rank_bubble(
    rank: &mut Vec<usize>,
    groups: &Vec<Vec<usize>>,
    from_up: bool,
    heaviest_g_idx: usize,
    interactor: &mut Interactor,
) -> bool {
    let order = if from_up {
        (0..heaviest_g_idx).rev().collect::<Vec<usize>>()
    } else {
        (0..heaviest_g_idx).collect::<Vec<usize>>()
    };
    for i in order {
        match interactor.output_query(&groups[rank[i]], &groups[rank[i + 1]]) {
            BalanceResult::Left => break,                // <
            BalanceResult::Right => rank.swap(i, i + 1), // >
            BalanceResult::Equal => break,               // =
            BalanceResult::Unknown => return false,
        }
    }
    true
}

fn update_rank_binary_search(
    rank: &mut Vec<usize>,
    groups: &Vec<Vec<usize>>,
    from_up: bool,
    heaviest_g_idx: usize,
    interactor: &mut Interactor,
) -> bool {
    let move_g_idx = if from_up { heaviest_g_idx } else { 0 };
    let move_g = rank[move_g_idx];
    rank.remove(move_g_idx);
    let mut l = -1 as i32;
    let mut r = rank.len() as i32;
    while r - l > 1 {
        let m = (l + r) / 2;
        match interactor.output_query(&groups[rank[m as usize]], &groups[move_g]) {
            BalanceResult::Left | BalanceResult::Equal => l = m, // <
            BalanceResult::Right => r = m,                       // >
            BalanceResult::Unknown => {
                rank.insert(move_g_idx, move_g);
                return false;
            }
        }
    }
    rank.insert(r as usize, move_g);
    true
}

fn solve(input: &Input, interactor: &mut Interactor) {
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
    while interactor.query_count < input.q {
        // TODO: ロールバックの高速化
        let copied_groups = groups.clone();
        let mut heaviest_g_idx = input.d - 1;
        while groups[rank[heaviest_g_idx]].len() == 1 {
            heaviest_g_idx -= 1;
        }
        trial_count += 1;
        if rnd::nextf() < 0.5 {
            let move_w_idx_in_group = rnd::gen_range(0, groups[rank[heaviest_g_idx]].len());
            let move_w_idx = groups[rank[heaviest_g_idx]][move_w_idx_in_group];

            groups[rank[heaviest_g_idx]].swap_remove(move_w_idx_in_group);
            if interactor.output_query(&groups[rank[0]], &groups[rank[heaviest_g_idx]])
                == BalanceResult::Right
            {
                groups[rank[heaviest_g_idx]].push(move_w_idx);
                continue;
            }
            move_adopted_count += 1;
            eprintln!("[{} / {}] adopt_move", interactor.query_count, input.q);
            if !update_rank(&mut rank, &groups, true, heaviest_g_idx, input, interactor) {
                groups = copied_groups;
                continue;
            }
            groups[rank[0]].push(move_w_idx);
            if !update_rank(&mut rank, &groups, false, heaviest_g_idx, input, interactor) {
                groups = copied_groups;
                continue;
            }
        } else {
            let (g_a, g_b) = (rank[0], rank[heaviest_g_idx]);
            let item_idx_in_group_a = rnd::gen_range(0, groups[g_a].len());
            let item_idx_in_group_b = rnd::gen_range(0, groups[g_b].len());
            let item_idx_a = groups[g_a][item_idx_in_group_a];
            let item_idx_b = groups[g_b][item_idx_in_group_b];
            if interactor.output_query(&vec![item_idx_a], &vec![item_idx_b]) == BalanceResult::Left
            {
                groups[g_a].swap_remove(item_idx_in_group_a);
                groups[g_b].swap_remove(item_idx_in_group_b);
                match interactor.output_query(&groups[g_a], &groups[g_b]) {
                    BalanceResult::Right | BalanceResult::Unknown => {
                        groups[g_a].push(item_idx_a);
                        groups[g_b].push(item_idx_b);
                    }
                    _ => {
                        swap_adopted_count += 1;
                        eprintln!("[{} / {}] adopt_swap", interactor.query_count, input.q);
                        groups[g_b].push(item_idx_a);
                        if !update_rank(&mut rank, &groups, true, heaviest_g_idx, input, interactor)
                        {
                            groups = copied_groups;
                            continue;
                        }
                        groups[g_a].push(item_idx_b);
                        if !update_rank(
                            &mut rank,
                            &groups,
                            false,
                            heaviest_g_idx,
                            input,
                            interactor,
                        ) {
                            groups = copied_groups;
                            continue;
                        }
                    }
                }
            }
        }

        let d = groups_to_output_d(&groups, input);
        interactor.output_d(&d, true);
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
