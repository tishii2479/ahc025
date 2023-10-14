mod def;
mod interactor;
mod util;

use crate::def::*;
use crate::interactor::*;
use crate::util::*;

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
            if interactor.query_count >= input.q {
                break;
            }

            interactor.output_query(&groups[pivot_g_idx], &groups[g_idx]);
            match interactor.read_result() {
                BalanceResult::Left => right_targets.push(g_idx), // <
                BalanceResult::Right => left_targets.push(g_idx), // >
                BalanceResult::Equal => right_targets.push(g_idx), // =
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
    let order = if from_up {
        (0..heaviest_g_idx).rev().collect::<Vec<usize>>()
    } else {
        (0..heaviest_g_idx).collect::<Vec<usize>>()
    };
    for i in order {
        if interactor.query_count >= input.q {
            return false;
        }
        interactor.output_query(&groups[rank[i]], &groups[rank[i + 1]]);
        match interactor.read_result() {
            BalanceResult::Left => break,                // <
            BalanceResult::Right => rank.swap(i, i + 1), // >
            BalanceResult::Equal => break,               // =
        }
    }
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

    // 一番重いグループから軽いグループに移す
    while interactor.query_count < input.q {
        // TODO: ロールバックの高速化
        let copied_groups = groups.clone();
        let mut heaviest_g_idx = input.d - 1;
        while groups[rank[heaviest_g_idx]].len() == 1 {
            heaviest_g_idx -= 1;
        }
        let move_w_idx_in_group = rnd::gen_range(0, groups[rank[heaviest_g_idx]].len());
        let move_w_idx = groups[rank[heaviest_g_idx]][move_w_idx_in_group];
        groups[rank[heaviest_g_idx]].swap_remove(move_w_idx_in_group);
        if !update_rank(&mut rank, &groups, true, heaviest_g_idx, input, interactor) {
            groups = copied_groups;
            break;
        }
        groups[rank[0]].push(move_w_idx);
        if !update_rank(&mut rank, &groups, false, heaviest_g_idx, input, interactor) {
            groups = copied_groups;
            break;
        }

        let d = groups_to_output_d(&groups, input);
        interactor.output_d(&d, true);
    }

    let d = groups_to_output_d(&groups, input);
    interactor.output_d(&d, false);
}

fn groups_to_output_d(groups: &Vec<Vec<usize>>, input: &Input) -> Vec<usize> {
    let mut d = vec![0; input.n];
    for (g_idx, group) in groups.iter().enumerate() {
        for e in group {
            d[*e] = g_idx;
        }
    }
    d
}

fn main() {
    time::start_clock();

    let mut interactor = Interactor::new();
    let input = interactor.read_input();

    solve(&input, &mut interactor);
}
