use proconio::*;

mod util;

use util::*;

struct Input {
    n: usize,
    d: usize,
    q: usize,
    w: Vec<i64>,
}

fn read_input() -> Input {
    input! {
        n: usize,
        d: usize,
        q: usize,
        w: [i64; n]
    }
    Input { n, d, q, w }
}

fn main() {
    const TIME_LIMIT: f64 = 1.;
    time::start_clock();
    let input = read_input();
    let mut groups = vec![];
    let mut weight_total = 0.;
    for i in 0..input.n {
        groups.push(i % input.d);
        weight_total += input.w[i] as f64;
    }
    let weight_mean = weight_total / input.d as f64;

    let start_temp: f64 = 1e6;
    let end_temp: f64 = 1e3;
    let mut iteration = 0;

    fn calc_score(groups: &Vec<usize>, input: &Input, weight_mean: f64) -> f64 {
        let mut weights = vec![0.; input.d];
        for i in 0..input.n {
            weights[groups[i]] += input.w[i] as f64;
        }
        let mut v = 0.;
        for i in 0..input.d {
            v += (weights[i] - weight_mean).powf(2.);
        }
        return 1. + (100. * (v / input.d as f64).sqrt());
    }

    while time::elapsed_seconds() < TIME_LIMIT {
        let progress = time::elapsed_seconds() / TIME_LIMIT;
        let current_score = calc_score(&groups, &input, weight_mean);
        let current_temp = start_temp.powf(1. - progress) * end_temp.powf(progress);

        if rnd::nextf() < 0.5 {
            let (i, j) = (rnd::gen_range(0, input.n), rnd::gen_range(0, input.n));
            groups.swap(i, j);
            let new_score = calc_score(&groups, &input, weight_mean);
            if ((current_score - new_score) / current_temp).exp() > rnd::nextf() {
                // adopt
            } else {
                groups.swap(i, j);
            }
        } else {
            let i = rnd::gen_range(0, input.n);
            let j = rnd::gen_range(0, input.d);
            let prev_j = groups[i];
            groups[i] = j;
            let new_score = calc_score(&groups, &input, weight_mean);
            if ((current_score - new_score) / current_temp).exp() > rnd::nextf() {
                // adopt
            } else {
                groups[i] = prev_j;
            }
        }
        iteration += 1;
        // if iteration % 10000 == 0 {
        //     eprintln!("[{:.3}] {}", time::elapsed_seconds(), current_score);
        // }
    }

    let mut result_json = "{".to_owned();
    result_json += format!("\"iteration\": {}, ", iteration).as_str();
    result_json += format!(
        "\"final_score\": {}, ",
        calc_score(&groups, &input, weight_mean).round() as i64
    )
    .as_str();
    result_json += "\"answer\": \"";
    for i in 0..input.n {
        result_json += format!("{} ", groups[i]).as_str();
    }
    result_json += "\"}";
    println!("{result_json}");
}
