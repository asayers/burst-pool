extern crate burst_pool;
extern crate env_logger;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
extern crate mio;
extern crate nix;
extern crate parking_lot_core;
extern crate rand;

mod basic;
mod unpark;
mod histogram;
// mod pipe_fanout;
mod pipe_ring;

fn main() {
    env_logger::init().unwrap();

    println!("# BASIC\n{}", basic::bench());
    println!("# UNPARK\n{}", unpark::bench());
    println!("# PIPE RING\n{}", pipe_ring::bench());
    // pipe_fanout::bench();   // FIXME
}
