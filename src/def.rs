#[derive(Debug)]
pub struct Input {
    pub n: usize,
    pub d: usize,
    pub q: usize,
}

#[derive(PartialEq, Eq)]
pub enum BalanceResult {
    Left,
    Right,
    Equal,
    Unknown,
}
