mod def;
mod interactor;
mod util;

use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;

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
    lighter_g_idx: usize,
    heaviest_g_idx: usize,
    input: &Input,
    interactor: &mut Interactor,
) -> bool {
    if input.d < 10 {
        update_rank_bubble(
            rank,
            groups,
            from_up,
            lighter_g_idx,
            heaviest_g_idx,
            interactor,
        )
    } else {
        update_rank_binary_search(
            rank,
            groups,
            from_up,
            lighter_g_idx,
            heaviest_g_idx,
            interactor,
        )
    }
}

fn update_rank_bubble(
    rank: &mut Vec<usize>,
    groups: &Vec<Vec<usize>>,
    from_up: bool,
    lighter_g_idx: usize,
    heavier_g_idx: usize,
    interactor: &mut Interactor,
) -> bool {
    let order = if from_up {
        (lighter_g_idx..heavier_g_idx).rev().collect::<Vec<usize>>()
    } else {
        (lighter_g_idx..heavier_g_idx).collect::<Vec<usize>>()
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
    lighter_g_idx: usize,
    heavier_g_idx: usize,
    interactor: &mut Interactor,
) -> bool {
    let move_g_idx = if from_up {
        heavier_g_idx
    } else {
        lighter_g_idx
    };
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

struct Balancer {
    hash_for_item: Vec<u64>,
    left_edges: HashMap<u64, Vec<u64>>,  // first <= second
    right_edges: HashMap<u64, Vec<u64>>, // first > second
}

impl Balancer {
    fn new(input: &Input) -> Balancer {
        let mut hash_for_item = vec![0; input.n];
        for i in 0..input.n {
            hash_for_item[i] = rnd::gen_range(0, usize::MAX) as u64;
        }
        Balancer {
            hash_for_item,
            left_edges: HashMap::new(),
            right_edges: HashMap::new(),
        }
    }

    fn get_result(
        &mut self,
        left_v: &Vec<usize>,
        right_v: &Vec<usize>,
        interactor: &mut Interactor,
    ) -> BalanceResult {
        let left_hash = self.to_hash(left_v);
        let right_hash = self.to_hash(right_v);

        let search_result = self.search_result(left_v, right_v);
        match search_result {
            BalanceResult::Unknown => {}
            BalanceResult::Left | BalanceResult::Equal => {
                add_edge(&mut self.left_edges, left_hash, right_hash);
                add_edge(&mut self.right_edges, right_hash, left_hash);
                return search_result;
            }
            BalanceResult::Right => {
                add_edge(&mut self.left_edges, right_hash, left_hash);
                add_edge(&mut self.right_edges, left_hash, right_hash);
                return search_result;
            }
        }
        let query_result = interactor.output_query(left_v, right_v);

        fn add_edge(edges: &mut HashMap<u64, Vec<u64>>, first_hash: u64, second_hash: u64) {
            if let Some(current_edges) = edges.get_mut(&first_hash) {
                if !current_edges.contains(&second_hash) {
                    current_edges.push(second_hash);
                }
            } else {
                edges.insert(first_hash, vec![second_hash]);
            }
        }

        match query_result {
            BalanceResult::Left | BalanceResult::Equal => {
                add_edge(&mut self.left_edges, left_hash, right_hash);
                add_edge(&mut self.right_edges, right_hash, left_hash);
                // add_edge(&mut self.left_edges, left_hash, right_hash);
                // add_edge(&mut self.right_edges, right_hash, left_hash);
            }
            BalanceResult::Right => {
                add_edge(&mut self.left_edges, right_hash, left_hash);
                add_edge(&mut self.right_edges, left_hash, right_hash);
            }
            BalanceResult::Unknown => {}
        }
        query_result
    }

    fn search_result(&self, left_v: &Vec<usize>, right_v: &Vec<usize>) -> BalanceResult {
        // NOTE: left = rightの時は稀（だと思う）ので、ここでは無視している
        let left_hash = self.to_hash(left_v);
        let right_hash = self.to_hash(right_v);

        fn is_reachable(edges: &HashMap<u64, Vec<u64>>, from_hash: u64, to_hash: u64) -> bool {
            let mut q = VecDeque::new();
            let mut seen = HashSet::new();
            q.push_back(from_hash);
            seen.insert(from_hash);
            while let Some(v) = q.pop_front() {
                if let Some(v_edges) = edges.get(&v) {
                    for u in v_edges {
                        if seen.contains(u) {
                            continue;
                        }
                        if *u == to_hash {
                            return true;
                        }
                        seen.insert(*u);
                        q.push_back(*u);
                    }
                }
            }
            false
        }

        if is_reachable(&self.left_edges, left_hash, right_hash) {
            return BalanceResult::Left;
        } else if is_reachable(&self.right_edges, left_hash, right_hash) {
            return BalanceResult::Right;
        }
        BalanceResult::Unknown
    }

    fn to_hash(&self, v: &Vec<usize>) -> u64 {
        let mut hash = 0;
        for e in v.iter() {
            hash ^= self.hash_for_item[*e];
        }
        hash
    }
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
        // let lighter_g_idx = 0;
        // let mut heavier_g_idx = input.d - 1;
        // // 重いグループが一個しかアイテムがなければ、グループを変更する
        // while groups[rank[heavier_g_idx]].len() == 1 {
        //     heavier_g_idx -= 1;
        // }

        trial_count += 1;
        if rnd::nextf() < 0.5 {
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
                groups[rank[heavier_g_idx]].push(item_idx);
                continue;
            }

            move_adopted_count += 1;
            eprintln!("[{} / {}] adopt_move", interactor.query_count, input.q);
            if !update_rank(
                &mut rank,
                &groups,
                true,
                lighter_g_idx,
                heavier_g_idx,
                input,
                interactor,
            ) {
                groups = copied_groups;
                continue;
            }
            groups[rank[lighter_g_idx]].push(item_idx);
            if !update_rank(
                &mut rank,
                &groups,
                false,
                lighter_g_idx,
                heavier_g_idx,
                input,
                interactor,
            ) {
                groups = copied_groups;
                continue;
            }
        } else {
            let item_idx_in_group_a = rnd::gen_range(0, groups[rank[lighter_g_idx]].len());
            let item_idx_in_group_b = rnd::gen_range(0, groups[rank[heavier_g_idx]].len());
            let item_idx_a = groups[rank[lighter_g_idx]][item_idx_in_group_a];
            let item_idx_b = groups[rank[heavier_g_idx]][item_idx_in_group_b];

            // 入れ替えようとしているアイテムの大小関係が集合の大小関係と一致しなければ不採用
            if balancer.get_result(&vec![item_idx_a], &vec![item_idx_b], interactor)
                != BalanceResult::Left
            {
                continue;
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
                    swap_adopted_count += 1;
                    eprintln!("[{} / {}] adopt_swap", interactor.query_count, input.q);
                    groups[rank[heavier_g_idx]].push(item_idx_a);
                    if !update_rank(
                        &mut rank,
                        &groups,
                        true,
                        lighter_g_idx,
                        heavier_g_idx,
                        input,
                        interactor,
                    ) {
                        groups = copied_groups;
                        continue;
                    }
                    groups[rank[lighter_g_idx]].push(item_idx_b);
                    if !update_rank(
                        &mut rank,
                        &groups,
                        false,
                        lighter_g_idx,
                        heavier_g_idx,
                        input,
                        interactor,
                    ) {
                        groups = copied_groups;
                        continue;
                    }
                }
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
