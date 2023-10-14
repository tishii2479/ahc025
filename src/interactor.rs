use crate::def::*;
use std::io::{Stdin, Write};

use proconio::*;

pub struct Interactor {
    source: proconio::source::line::LineSource<std::io::BufReader<Stdin>>,
    pub query_count: usize,
}

impl Interactor {
    pub fn new() -> Interactor {
        Interactor {
            source: proconio::source::line::LineSource::new(std::io::BufReader::new(
                std::io::stdin(),
            )),
            query_count: 0,
        }
    }

    pub fn read_input(&mut self) -> Input {
        input! {
            from &mut self.source,
            n: usize,
            d: usize,
            q: usize
        }
        Input { n, d, q }
    }

    pub fn output_query(&mut self, left_v: &Vec<usize>, right_v: &Vec<usize>) {
        self.query_count += 1;
        print!("{} {} ", left_v.len(), right_v.len());
        for e in left_v.iter() {
            print!("{} ", e);
        }
        for e in right_v.iter() {
            print!("{} ", e);
        }
        println!();
        self.flush();
    }

    pub fn read_result(&mut self) -> BalanceResult {
        input! {
            from &mut self.source,
            s: String
        }
        match s.as_str() {
            "<" => BalanceResult::Left,
            ">" => BalanceResult::Right,
            "=" => BalanceResult::Equal,
            _ => panic!("failed to read_result: {}", s),
        }
    }

    pub fn output_d(&self, d: &Vec<usize>, for_debug: bool) {
        if for_debug {
            print!("#c ");
        }
        for e in d.iter() {
            print!("{} ", e);
        }
        println!();
        self.flush();
    }

    fn flush(&self) {
        std::io::stdout().flush().unwrap();
    }
}
