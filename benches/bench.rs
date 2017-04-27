extern crate burst_pool;
extern crate env_logger;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
extern crate nix;
extern crate rand;

mod basic;
mod histogram;

fn main() {
    env_logger::init().unwrap();

    println!("# BASIC\n{}", basic::bench());
}
