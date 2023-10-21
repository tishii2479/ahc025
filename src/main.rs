mod action;
mod def;
mod interactor;
mod util;

use crate::action::*;
use crate::def::*;
use crate::interactor::*;
use crate::util::*;

fn select_g_idx_pair(input: &Input) -> (usize, usize) {
    const P: f64 = 0.3;
    let par =
        1 + (time::elapsed_seconds() * input.d as f64 / 3. / (TIME_LIMIT - 0.1)).round() as usize;
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
        let action = if p < 0.5 && time::elapsed_seconds() < 1.0 {
            action_move
        } else if p < 0.9 && time::elapsed_seconds() < 1.0 {
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
        }

        let d = groups_to_output_d(&groups, input);
        interactor.output_d(&d, true);
    }

    // 必要ないクエリを消化する
    if interactor.query_count < input.q {
        eprintln!("remaining query:     {}", input.q - interactor.query_count);
    }

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
