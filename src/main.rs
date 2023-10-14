mod def;
mod interactor;
mod util;

use crate::def::*;
use crate::interactor::*;
use crate::util::*;

fn main() {
    time::start_clock();

    let mut interactor = Interactor::new();
    let input = interactor.read_input();

    for _ in 0..input.q {
        interactor.output_query(&vec![0], &vec![1]);
    }
    interactor.output_d(&vec![0; input.n]);
}
