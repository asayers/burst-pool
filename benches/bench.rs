extern crate burst_pool;
extern crate deque;
extern crate env_logger;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
extern crate mio;
extern crate nix;
extern crate parking_lot_core;
extern crate rand;
extern crate scoped_threadpool;
extern crate threadpool;

// mod bench_parking_lot;
// mod pipe_fanout;
mod bench_burst_pool;
mod bench_deque;
mod bench_scoped_threadpool;
mod bench_threadpool;
mod bench_unpark;
// mod bench_unpark_worst;
mod histogram;
mod pipe_ring;

fn main() {
    env_logger::init().unwrap();

    // println!("# parking_lot\n{}", bench_parking_lot::bench());
    // println!("# pipe_fanout\n{}", pipe_fanout::bench());
    // println!("# unpark_worst\n{}", bench_unpark_worst::bench());
    println!("# burst_pool\n{}", bench_burst_pool::bench());
    println!("# pipe_ring\n{}", pipe_ring::bench());
    println!("# scoped_threadpool\n{}", bench_scoped_threadpool::bench());
    println!("# threadpool\n{}", bench_threadpool::bench());
    println!("# unpark\n{}", bench_unpark::bench());
    println!("# deque\n{}", bench_deque::bench());
}
