use std::fmt::{self, Display, Formatter};
use std::time::*;

pub struct Stats(Vec<u32>);

impl Stats {
    pub fn new(times: &[Duration]) -> Stats {
        let mut times: Vec<u32> = times.iter().map(Duration::subsec_nanos).collect();
        times.sort();
        Stats(times)
    }

    fn percentile(&self, x: usize) -> u32 {
        self.0[(x * (self.0.len() - 1)) / 100]
    }

    fn mean_stddev(&self) -> (f64, f64) {
        let n = self.0.len() as f64;
        let xbar = self.0.iter().sum::<u32>() as f64 / n;
        let sd = (self.0.iter().map(|&x|(x as f64 - xbar).powi(2)).sum::<f64>() / (n - 1.0)).sqrt();
        (xbar, sd)
    }
}

impl Display for Stats {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let (avg, stddev) = self.mean_stddev();
        write!(f, "\
avg {:.0} ns (stddev {:.0} ns)
  0 {:>5} ns
  1 {:>5} ns
 10 {:>5} ns
 25 {:>5} ns
 50 {:>5} ns
 75 {:>5} ns
 90 {:>5} ns
 99 {:>5} ns
100 {:>5} ns
",
            avg, stddev,
            self.percentile(  0),
            self.percentile(  1),
            self.percentile( 10),
            self.percentile( 25),
            self.percentile( 50),
            self.percentile( 75),
            self.percentile( 90),
            self.percentile( 99),
            self.percentile(100),
            )
    }
}
