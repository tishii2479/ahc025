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
