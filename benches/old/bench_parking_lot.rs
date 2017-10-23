use histogram::Histogram;
use mio::*;
use mio::unix::EventedFd;
use nix::unistd::{pipe,write,read};
use parking_lot_core::{ParkToken, UnparkToken};
use std::io;
use std::os::unix::io::RawFd;
use std::sync::Mutex;
use std::sync::mpsc;
use std::thread;
use std::time::*;
use std::time::*;

const THREADS: usize = 10;

pub fn bench() -> Histogram {
    env_logger::init().unwrap();

    let (chan_tx, chan_rx) = mpsc::channel();

    info!("Spawning workers...");
    for key in 0..THREADS {
        let chan_tx = chan_tx.clone();
        thread::spawn(move|| worker(key, chan_tx).expect("thread died"));
    }
    thread::sleep_ms(1000);

    loop {
        let start = Instant::now();
        for key in 0..THREADS {
            unsafe { parking_lot_core::unpark_all(key, UnparkToken(0)); }
        }
        thread::sleep_ms(200);
        for _ in 0..THREADS {
            let ts = chan_rx.recv().unwrap();
            println!("{}", ts.duration_since(start).subsec_nanos() / 1000);
        }
    }
}

fn worker(key: usize, chan: mpsc::Sender<Instant>) -> Result<(),io::Error> {
    info!("Started worker");
    loop {
        unsafe { parking_lot_core::park(key, || true, || (), |_,_| (), ParkToken(0), None); }
        let ts = Instant::now();
        thread::sleep_ms(50);
        chan.send(ts);
    }
}
