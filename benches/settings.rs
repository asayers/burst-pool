
#[derive(Debug, Clone)]
pub struct BenchSpec {
    pub num_receivers: usize,
    pub num_msgs: usize,
    pub iters: usize,
    pub wait_ms: u64,
}

pub const BENCH_SPECS: &[BenchSpec] = &[
    BenchSpec {
        num_receivers: 3,
        num_msgs: 2,
        iters: 500,
        wait_ms: 50,
    },
    BenchSpec {
        num_receivers: 3,
        num_msgs: 3,
        iters: 500,
        wait_ms: 50,
    },
    BenchSpec {
        num_receivers: 3,
        num_msgs: 5,
        iters: 500,
        wait_ms: 50,
    },
    BenchSpec {
        num_receivers: 6,
        num_msgs: 6,
        iters: 500,
        wait_ms: 50,
    },
];
