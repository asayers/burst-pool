extern crate burst_pool;
extern crate env_logger;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
extern crate mio;
extern crate nix;
extern crate parking_lot_core;
extern crate rand;

// mod bench_parking_lot;
// mod pipe_fanout;
mod bench_burst_pool;
mod bench_unpark;
mod histogram;
mod pipe_ring;

fn main() {
    env_logger::init().unwrap();

    // println!("# parking_lot\n{}", bench_parking_lot::bench());
    // println!("# pipe_fanout\n{}", pipe_fanout::bench());
    println!("# burst_pool\n{}", bench_burst_pool::bench());
    println!("# pipe_ring\n{}", pipe_ring::bench());
    println!("# unpark\n{}", bench_unpark::bench());
}
