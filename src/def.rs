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
    pub left_edges: FastHashMap<u128, Vec<u128>>, // first <= second
    pub right_edges: FastHashMap<u128, Vec<u128>>, // first > second
}

impl Balancer {
    pub fn new() -> Balancer {
        Balancer {
            left_edges: FastHashMap::default(),
            right_edges: FastHashMap::default(),
        }
    }

    ///
    /// 1. 部分集合が存在するかチェックし、存在するなら辺を引く
    /// 2. 差分が1個の集合が存在するかチェックし、存在し、かつ差分の大小関係がわかっているものに対して辺を引く
    /// 3. 元の位置から探索を開始する
    ///
    pub fn get_result(
        &mut self,
        left_v: &Vec<usize>,
        right_v: &Vec<usize>,
        interactor: &mut Interactor,
    ) -> BalanceResult {
        let check_empty_result = self.check_empty_comparison(left_v, right_v);
        if check_empty_result != BalanceResult::Unknown {
            return check_empty_result;
        }
        assert!(left_v.len() > 0 && right_v.len() > 0);

        let left_hash = self.to_hash(left_v);
        let right_hash = self.to_hash(right_v);

        self.add_additional_edges(left_hash);
        self.add_additional_edges(right_hash);

        let search_result = self.search_result(left_hash, right_hash);
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

        match query_result {
            BalanceResult::Left | BalanceResult::Equal => {
                add_edge(&mut self.left_edges, left_hash, right_hash);
                add_edge(&mut self.right_edges, right_hash, left_hash);
            }
            BalanceResult::Right => {
                add_edge(&mut self.left_edges, right_hash, left_hash);
                add_edge(&mut self.right_edges, left_hash, right_hash);
            }
            BalanceResult::Unknown => {}
        }
        query_result
    }

    fn search_result(&self, left_hash: u128, right_hash: u128) -> BalanceResult {
        // NOTE: left = rightの時は稀（だと思う）ので、ここでは無視している
        const MAX_DEPTH: u16 = 5;

        fn is_reachable(
            edges: &FastHashMap<u128, Vec<u128>>,
            from_hash: u128,
            to_hash: u128,
        ) -> bool {
            let mut q = Queue::default();
            let mut seen = FastHashSet::default();
            q.push_back((from_hash, 0));
            seen.insert(from_hash);
            while let Some((v, depth)) = q.pop_front() {
                let Some(v_edges) = edges.get(&v) else { continue };
                for u in v_edges {
                    if seen.contains(u) {
                        continue;
                    }
                    if *u == to_hash {
                        return true;
                    }
                    if depth >= MAX_DEPTH {
                        continue;
                    }
                    seen.insert(*u);
                    q.push_back((*u, depth + 1));
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

    pub fn find_lighter_in_group(&self, v: usize, groups: &Vec<usize>) -> usize {
        let mut q = vec![];
        let mut seen = FastHashSet::default();
        let group_hash = self.to_hash(groups);
        let v_hash = 1 << v;
        q.push((v_hash, 0));
        seen.insert(v_hash);

        let mut lightest_v = v_hash;
        let mut lightest_depth = 0;

        while let Some((v, depth)) = q.pop() {
            let Some(edges) = self.right_edges.get(&v) else { continue };
            for u in edges {
                if seen.contains(u) {
                    continue;
                }
                if u.count_ones() == 1 && (u & group_hash) > 0 && depth > lightest_depth {
                    lightest_depth = depth;
                    lightest_v = *u;
                }
                q.push((*u, depth + 1));
                seen.insert(*u);
            }
        }

        lightest_v.trailing_zeros() as usize
    }

    ///
    /// 1. 部分集合が存在するかチェックし、存在するなら辺を引く
    /// 2. 差分が1個の集合が存在するかチェックし、存在し、かつ差分の大小関係がわかっているものに対して辺を引く
    ///
    fn add_additional_edges(&mut self, v_hash: u128) {
        let mut additional_edges = vec![]; // first < second
        let edge_data = [&self.left_edges, &self.right_edges];
        for edges in edge_data {
            if edges.contains_key(&v_hash) {
                continue;
            }
            for (u_hash, _) in edges.iter() {
                // 部分集合のチェック
                if (v_hash & *u_hash) == *u_hash {
                    additional_edges.push((*u_hash, v_hash));
                    continue;
                }
                if (v_hash & *u_hash) == v_hash {
                    additional_edges.push((v_hash, *u_hash));
                    continue;
                }

                // 差分が1個のものをチェック
                // 包含しているパターンは前まででチェックできている
                // v = 010111
                // u = 001111
                // v ^ u = 011000
                // a = v & u = 000111
                // v ^ a = 010000, u ^ a = 001000
                if (v_hash ^ *u_hash).count_ones() == 2 {
                    let a = v_hash & *u_hash;
                    match self.search_result(v_hash ^ a, *u_hash ^ a) {
                        BalanceResult::Left | BalanceResult::Equal => {
                            additional_edges.push((v_hash, *u_hash));
                        }
                        BalanceResult::Right => {
                            additional_edges.push((*u_hash, v_hash));
                        }
                        _ => {}
                    }
                }
            }
        }

        // NOTE: add_edgeで重複は考慮されるので、ここで取り除く必要はない
        for (left_hash, right_hash) in additional_edges {
            add_edge(&mut self.left_edges, left_hash, right_hash);
            add_edge(&mut self.right_edges, right_hash, left_hash);
        }
    }

    fn check_empty_comparison(&self, left_v: &Vec<usize>, right_v: &Vec<usize>) -> BalanceResult {
        if left_v.len() == 0 && right_v.len() > 0 {
            return BalanceResult::Left;
        } else if left_v.len() > 0 && right_v.len() == 0 {
            return BalanceResult::Right;
        } else if left_v.len() == 0 && right_v.len() == 0 {
            return BalanceResult::Equal;
        }
        BalanceResult::Unknown
    }

    fn to_hash(&self, v: &Vec<usize>) -> u128 {
        let mut hash = 0;
        for e in v.iter() {
            hash |= self.hash(*e);
        }
        hash
    }

    fn hash(&self, v: usize) -> u128 {
        1 << v
    }

    #[allow(unused)]
    fn print_edges(&self) {
        let mut edges_mean = 0.;
        for (_, edges) in self.left_edges.iter() {
            edges_mean += edges.len() as f64;
        }
        edges_mean /= self.left_edges.len() as f64;
        eprintln!(
            "{:?} {:?} {} {} {}",
            self.left_edges,
            self.right_edges,
            self.left_edges.len(),
            self.right_edges.len(),
            edges_mean
        );
    }
}

fn add_edge(edges: &mut FastHashMap<u128, Vec<u128>>, first_hash: u128, second_hash: u128) {
    if let Some(current_edges) = edges.get_mut(&first_hash) {
        if !current_edges.contains(&second_hash) {
            current_edges.push(second_hash);
        }
    } else {
        edges.insert(first_hash, vec![second_hash]);
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
    const PAR: usize = 8; // :param
    if input.d < PAR {
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
    let mut l = lighter_g_idx as i32 - 1;
    let mut r = heavier_g_idx as i32;
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
