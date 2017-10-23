use std::fmt::{self, Display, Formatter};
use std::time::*;

pub struct Stats {
    avg: f64,
    stddev: f64,
    med: u32,
    best: u32,
    worst: u32,
}

pub fn mk_stats(times: &[Duration]) -> Stats {
    let mut times: Vec<u32> = times.iter().map(Duration::subsec_nanos).collect();
    times.sort();
    let n = times.len() as f64;
    let xbar = times.iter().sum::<u32>() as f64 / n;
    let sd = (times.iter().map(|&x|(x as f64 - xbar).powi(2)).sum::<f64>() / (n - 1.0)).sqrt();

    Stats {
        avg: xbar,
        stddev: sd,
        med: times[times.len() / 2],
        best: times[0],
        worst: times[times.len() - 1],
    }
}

impl Display for Stats {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "\
avg {:.0} ns (stddev {:.0} ns)
med {:>5} ns (range {}..{})",
            self.avg, self.stddev, self.med, self.best, self.worst)
    }
}
