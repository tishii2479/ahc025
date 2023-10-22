use crate::def::*;
use crate::interactor::*;
use crate::util::*;

fn select_lighter_item(group: &mut Vec<usize>, balancer: &mut Balancer) -> usize {
    let item_idx_in_group = rnd::gen_range(0, group.len());
    let item_idx = group[item_idx_in_group];
    let item_idx = balancer.find_lighter_in_group(item_idx, group);
    item_idx
}

pub fn action_move(
    heavier_g_idx: usize,
    lighter_g_idx: usize,
    groups: &mut Vec<Vec<usize>>,
    rank: &mut Vec<usize>,
    input: &Input,
    balancer: &mut Balancer,
    interactor: &mut Interactor,
) -> bool {
    let item_idx = select_lighter_item(&mut groups[rank[heavier_g_idx]], balancer);
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

    let prev_g_idx = rank[heavier_g_idx];
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
        groups[prev_g_idx].push(item_idx);
        // rankは更新できなかったが、moveは成功している
        return true;
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
        // rankは更新できなかったが、moveは成功している
        return true;
    }

    true
}

pub fn action_swap(
    heavier_g_idx: usize,
    lighter_g_idx: usize,
    groups: &mut Vec<Vec<usize>>,
    rank: &mut Vec<usize>,
    input: &Input,
    balancer: &mut Balancer,
    interactor: &mut Interactor,
) -> bool {
    let item_idx_a = select_lighter_item(&mut groups[rank[lighter_g_idx]], balancer);
    let item_idx_b = select_lighter_item(&mut groups[rank[heavier_g_idx]], balancer);

    // 入れ替えようとしているアイテムの大小関係が集合の大小関係と一致しなければ不採用
    match balancer.get_result(&vec![item_idx_a], &vec![item_idx_b], interactor) {
        BalanceResult::Left | BalanceResult::Equal => {}
        _ => return false,
    }

    let i = groups[rank[lighter_g_idx]]
        .iter()
        .position(|x| *x == item_idx_a)
        .unwrap();
    groups[rank[lighter_g_idx]].swap_remove(i);
    let i = groups[rank[heavier_g_idx]]
        .iter()
        .position(|x| *x == item_idx_b)
        .unwrap();
    groups[rank[heavier_g_idx]].swap_remove(i);
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
            let to_idx = rank[lighter_g_idx];
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
                // 最後に更新できなかった場合、変更先に更新する
                groups[to_idx].push(item_idx_b);
                // rankは更新できなかったが、swapは成功している
                return true;
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
                // rankは更新できなかったが、swapは成功している
                return true;
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
/// NOTE: trial_count = 0にすればaction_swapと一緒の挙動？
///
pub fn action_swap2(
    heavier_g_idx: usize,
    lighter_g_idx: usize,
    groups: &mut Vec<Vec<usize>>,
    rank: &mut Vec<usize>,
    input: &Input,
    balancer: &mut Balancer,
    interactor: &mut Interactor,
) -> bool {
    const TRIAL_COUNT: usize = 3;
    let mut item_indices_a =
        vec![groups[rank[lighter_g_idx]][rnd::gen_range(0, groups[rank[lighter_g_idx]].len())]];
    let mut item_indices_b =
        vec![groups[rank[heavier_g_idx]][rnd::gen_range(0, groups[rank[heavier_g_idx]].len())]];

    // 入れ替えようとしているアイテムの大小関係が集合の大小関係と一致しなければ不採用
    match balancer.get_result(&item_indices_a, &item_indices_b, interactor) {
        BalanceResult::Right => {
            // 重い方に大小関係が入れ替わるものがあれば足す
            for _ in 0..TRIAL_COUNT {
                let b2 = select_lighter_item(&mut groups[rank[heavier_g_idx]], balancer);
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
            for _ in 0..TRIAL_COUNT {
                let a2 = select_lighter_item(&mut groups[rank[lighter_g_idx]], balancer);
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
            let to_idx = rank[lighter_g_idx];
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
                    groups[to_idx].push(*item_idx_b);
                }
                // rankは更新できなかったが、swapは成功している
                return true;
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
                // rankは更新できなかったが、swapは成功している
                return true;
            }
            if item_indices_a.len() > 1 || item_indices_b.len() > 1 {
                eprintln!("swap2: {:?} {:?}", item_indices_a, item_indices_b);
            }
            return true;
        }
    }
}
