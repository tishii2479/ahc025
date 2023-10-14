#[derive(Debug)]
pub struct Input {
    pub n: usize,
    pub d: usize,
    pub q: usize,
}

pub enum BalanceResult {
    Left,
    Right,
    Equal,
}
