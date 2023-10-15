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
