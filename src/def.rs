use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;

pub const TIME_LIMIT: f64 = 1.9;

use crate::interactor::*;
use crate::util::*;

#[derive(Debug)]
pub struct Input {
    pub n: usize,
    pub d: usize,
    pub q: usize,
}

#[derive(PartialEq, Eq, Debug)]
pub enum BalanceResult {
    Left,    // <
    Right,   // >
    Equal,   // =
    Unknown, // failed to get result (query limit or search failure)
}

pub struct Balancer {
    hash_for_item: Vec<u64>,
    left_edges: HashMap<u64, Vec<u64>>,  // first <= second
    right_edges: HashMap<u64, Vec<u64>>, // first > second
}

impl Balancer {
    pub fn new(input: &Input) -> Balancer {
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

    pub fn get_result(
        &mut self,
        left_v: &Vec<usize>,
        right_v: &Vec<usize>,
        interactor: &mut Interactor,
    ) -> BalanceResult {
        if left_v.len() == 0 && right_v.len() > 0 {
            return BalanceResult::Left;
        } else if left_v.len() > 0 && right_v.len() == 0 {
            return BalanceResult::Right;
        } else if left_v.len() == 0 && right_v.len() == 0 {
            return BalanceResult::Equal;
        }
        assert!(left_v.len() > 0 && right_v.len() > 0);

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
            // TODO: 再確保が起こらないようにする
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

pub fn groups_to_output_d(groups: &Vec<Vec<usize>>, input: &Input) -> Vec<usize> {
    let mut d = vec![0; input.n];
    for (g_idx, group) in groups.iter().enumerate() {
        for e in group {
            d[*e] = g_idx;
        }
    }
    d
}

pub fn sort_groups(
    groups: &Vec<Vec<usize>>,
    input: &Input,
    interactor: &mut Interactor,
    balancer: &mut Balancer,
) -> Vec<usize> {
    fn q_sort(
        targets: Vec<usize>,
        groups: &Vec<Vec<usize>>,
        input: &Input,
        interactor: &mut Interactor,
        balancer: &mut Balancer,
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
            match balancer.get_result(&groups[pivot_g_idx], &groups[g_idx], interactor) {
                BalanceResult::Left => right_targets.push(g_idx), // <
                BalanceResult::Right => left_targets.push(g_idx), // >
                BalanceResult::Equal => right_targets.push(g_idx), // =
                BalanceResult::Unknown => break,
            }
        }
        [
            q_sort(left_targets, groups, input, interactor, balancer),
            q_sort(right_targets, groups, input, interactor, balancer),
        ]
        .concat()
    }

    q_sort(
        (0..groups.len()).collect(),
        groups,
        input,
        interactor,
        balancer,
    )
}

pub fn update_rank(
    rank: &mut Vec<usize>,
    groups: &Vec<Vec<usize>>,
    from_up: bool,
    lighter_g_idx: usize,
    heaviest_g_idx: usize,
    input: &Input,
    interactor: &mut Interactor,
    balancer: &mut Balancer,
) -> bool {
    if input.d < 10 {
        update_rank_linear_search(
            rank,
            groups,
            from_up,
            lighter_g_idx,
            heaviest_g_idx,
            interactor,
            balancer,
        )
    } else {
        update_rank_binary_search(
            rank,
            groups,
            from_up,
            lighter_g_idx,
            heaviest_g_idx,
            interactor,
            balancer,
        )
    }
}

pub fn update_rank_linear_search(
    rank: &mut Vec<usize>,
    groups: &Vec<Vec<usize>>,
    from_up: bool,
    lighter_g_idx: usize,
    heavier_g_idx: usize,
    interactor: &mut Interactor,
    balancer: &mut Balancer,
) -> bool {
    let order = if from_up {
        (lighter_g_idx..heavier_g_idx).rev().collect::<Vec<usize>>()
    } else {
        (lighter_g_idx..heavier_g_idx).collect::<Vec<usize>>()
    };
    for i in order {
        match balancer.get_result(&groups[rank[i]], &groups[rank[i + 1]], interactor) {
            BalanceResult::Left => break,                // <
            BalanceResult::Right => rank.swap(i, i + 1), // >
            BalanceResult::Equal => break,               // =
            BalanceResult::Unknown => return false,
        }
    }
    true
}

pub fn update_rank_binary_search(
    rank: &mut Vec<usize>,
    groups: &Vec<Vec<usize>>,
    from_up: bool,
    lighter_g_idx: usize,
    heavier_g_idx: usize,
    interactor: &mut Interactor,
    balancer: &mut Balancer,
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
        match balancer.get_result(&groups[rank[m as usize]], &groups[move_g], interactor) {
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
