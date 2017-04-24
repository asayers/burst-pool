use std::fmt;

const BUCKETS: usize = 20;
const FACTOR: f64 = 1.5;

pub struct Histogram {
    buckets: [usize;BUCKETS],
    sum: f64,
}

impl Histogram {
    pub fn new() -> Histogram {
        Histogram {
            buckets: [0;BUCKETS],
            sum: 0.0,
        }
    }

    pub fn add(&mut self, x: f64) {
        let idx = ::std::cmp::min(x.log(FACTOR).floor() as usize, BUCKETS - 1);
        self.buckets[idx] += 1;
        self.sum += x;
    }

    pub fn clear(&mut self) {
        self.buckets = [0;BUCKETS];
        self.sum = 0.0;
    }
}

impl fmt::Display for Histogram {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let len = self.buckets.len();
        let mk_bar = |x| { vec!['+';x].into_iter().collect::<String>() };
        for i in 0..(len - 1) {
            writeln!(f, "{:>7.0}: {:>5} {}",
                     FACTOR.powi(i as i32), self.buckets[i], mk_bar(self.buckets[i]/5))?;
        }
        writeln!(f, "{:>6.0}+: {:>5} {}",
                 FACTOR.powi((len - 2) as i32), self.buckets[len - 1],
                 mk_bar(self.buckets[len - 1]/5))?;
        let n = self.buckets.iter().sum::<usize>();
        writeln!(f, "      ({:.1} mean)", self.sum / n as f64)?;
        writeln!(f, "      ({} total)", n)
    }
}
