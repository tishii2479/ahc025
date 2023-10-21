fn solve2(input: &Input, interactor: &mut Interactor) {
    // ランダムにグループに割り振る
    let mut groups = vec![vec![]; input.d];
    for i in 0..input.n {
        groups[i % input.d].push(i);
    }

    while interactor.query_count < input.q {
        let (mut g_a, mut g_b) = (rnd::gen_range(0, input.d), rnd::gen_range(0, input.d));
        if g_a == g_b {
            continue;
        }
        match interactor.output_query(&groups[g_a], &groups[g_b]) {
            BalanceResult::Left => {}
            BalanceResult::Equal => continue,
            BalanceResult::Right => std::mem::swap(&mut g_a, &mut g_b),
            BalanceResult::Unknown => continue,
        }

        // weight[g_a] < weight[g_b]
        for _ in 0..5 {
            if rnd::nextf() < 0.5 {
                if groups[g_b].len() == 1 {
                    continue;
                }
                let item_idx_in_group = rnd::gen_range(0, groups[g_b].len());
                let item_idx = groups[g_b][item_idx_in_group];
                groups[g_b].swap_remove(item_idx_in_group);
                groups[g_a].push(item_idx);

                match interactor.output_query(&groups[g_a], &groups[g_b]) {
                    BalanceResult::Right | BalanceResult::Unknown => {
                        groups[g_a].pop();
                        groups[g_b].push(item_idx);
                    }
                    _ => break,
                }
            } else {
                let item_idx_in_group_a = rnd::gen_range(0, groups[g_a].len());
                let item_idx_in_group_b = rnd::gen_range(0, groups[g_b].len());
                let item_idx_a = groups[g_a][item_idx_in_group_a];
                let item_idx_b = groups[g_b][item_idx_in_group_b];
                match interactor.output_query(&vec![item_idx_a], &vec![item_idx_b]) {
                    BalanceResult::Left => {
                        groups[g_a].swap_remove(item_idx_in_group_a);
                        groups[g_b].swap_remove(item_idx_in_group_b);
                        groups[g_a].push(item_idx_b);
                        groups[g_b].push(item_idx_a);
                        match interactor.output_query(&groups[g_a], &groups[g_b]) {
                            BalanceResult::Right | BalanceResult::Unknown => {
                                groups[g_a].pop();
                                groups[g_b].pop();
                                groups[g_a].push(item_idx_a);
                                groups[g_b].push(item_idx_b);
                            }
                            _ => break,
                        }
                    }
                    _ => break,
                }
            }
        }

        let d = groups_to_output_d(&groups, input);
        interactor.output_d(&d, true);
    }

    let d = groups_to_output_d(&groups, input);
    interactor.output_d(&d, false);
}

fn action_swap3(
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
    let mut item_indices_in_a;
    let mut item_indices_in_b;
    let mut item_indices_a;
    let mut item_indices_b;
    loop {
        item_indices_a = vec![];
        item_indices_b = vec![];
        item_indices_in_a = vec![];
        item_indices_in_b = vec![];
        for (i, item_idx) in groups[rank[lighter_g_idx]].iter().enumerate() {
            if rnd::nextf() < 1. / groups[rank[lighter_g_idx]].len() as f64 {
                item_indices_a.push(*item_idx);
                item_indices_in_a.push(i);
            }
        }
        for (i, item_idx) in groups[rank[heavier_g_idx]].iter().enumerate() {
            if rnd::nextf() < 1. / groups[rank[heavier_g_idx]].len() as f64 {
                item_indices_b.push(*item_idx);
                item_indices_in_b.push(i);
            }
        }
        if item_indices_a.len() > 0 && item_indices_b.len() > 0 {
            break;
        }
    }

    // 入れ替えようとしているアイテムの大小関係が集合の大小関係と一致しなければ不採用
    match balancer.get_result(&item_indices_a, &item_indices_b, interactor) {
        // 重い方に大小関係が入れ替わるものがあれば足す
        BalanceResult::Right => return false,
        BalanceResult::Left => {
            // 軽い方に足せるものがあれば足す
            for _ in 0..trial_count {
                if rnd::nextf() < 0.5 || item_indices_b.len() == 1 {
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
                } else {
                    let b2 = select_lighter_item(&item_indices_b, balancer);
                    item_indices_b
                        .swap_remove(item_indices_b.iter().position(|x| *x == b2).unwrap());
                    match balancer.get_result(&item_indices_a, &item_indices_b, interactor) {
                        BalanceResult::Left | BalanceResult::Equal => continue,
                        _ => {
                            item_indices_b.push(b2);
                        }
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

fn action_swap3(
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
    let mut item_indices_in_a;
    let mut item_indices_in_b;
    let mut item_indices_a;
    let mut item_indices_b;
    loop {
        item_indices_a = vec![];
        item_indices_b = vec![];
        item_indices_in_a = vec![];
        item_indices_in_b = vec![];
        for (i, item_idx) in groups[rank[lighter_g_idx]].iter().enumerate() {
            if rnd::nextf() < 1. / groups[rank[lighter_g_idx]].len() as f64 {
                item_indices_a.push(*item_idx);
                item_indices_in_a.push(i);
            }
        }
        for (i, item_idx) in groups[rank[heavier_g_idx]].iter().enumerate() {
            if rnd::nextf() < 1. / groups[rank[heavier_g_idx]].len() as f64 {
                item_indices_b.push(*item_idx);
                item_indices_in_b.push(i);
            }
        }
        if item_indices_a.len() > 0 && item_indices_b.len() > 0 {
            break;
        }
    }

    // 入れ替えようとしているアイテムの大小関係が集合の大小関係と一致しなければ不採用
    match balancer.get_result(&item_indices_a, &item_indices_b, interactor) {
        // 重い方に大小関係が入れ替わるものがあれば足す
        BalanceResult::Right => return false,
        BalanceResult::Left => {
            // 軽い方に足せるものがあれば足す
            for _ in 0..trial_count {
                if rnd::nextf() < 0.5 || item_indices_b.len() == 1 {
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
                } else {
                    let b2 = select_lighter_item(&item_indices_b, balancer);
                    item_indices_b
                        .swap_remove(item_indices_b.iter().position(|x| *x == b2).unwrap());
                    match balancer.get_result(&item_indices_a, &item_indices_b, interactor) {
                        BalanceResult::Left | BalanceResult::Equal => continue,
                        _ => {
                            item_indices_b.push(b2);
                        }
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
