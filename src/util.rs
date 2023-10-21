#![allow(unused)]

pub mod rnd {
    static mut S: usize = 88172645463325252;

    #[inline]
    pub fn next() -> usize {
        unsafe {
            S = S ^ S << 7;
            S = S ^ S >> 9;
            S
        }
    }

    #[inline]
    pub fn nextf() -> f64 {
        (next() & 4294967295) as f64 / 4294967296.
    }

    #[inline]
    pub fn gen_range(low: usize, high: usize) -> usize {
        assert!(low < high);
        (next() % (high - low)) + low
    }
}

pub mod time {
    static mut START: f64 = -1.;
    pub fn start_clock() {
        let _ = elapsed_seconds();
    }

    #[inline]
    pub fn elapsed_seconds() -> f64 {
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        unsafe {
            if START < 0. {
                START = t;
            }
            t - START
        }
    }
}

#[derive(Default)]
pub struct Queue {
    st_in: Vec<(u128, u16)>,
    st_out: Vec<(u128, u16)>,
}

impl Queue {
    pub fn push_back(&mut self, v: (u128, u16)) {
        self.st_in.push(v);
    }

    pub fn pop_front(&mut self) -> Option<(u128, u16)> {
        if self.st_out.is_empty() {
            if self.st_in.is_empty() {
                return None;
            }
            while let Some(v) = self.st_in.pop() {
                self.st_out.push(v);
            }
        }
        self.st_out.pop()
    }
}
