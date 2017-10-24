extern crate burst_pool;
extern crate pbr;
mod stats;

use burst_pool::*;
use stats::*;
use std::sync::*;
use std::thread;
use std::time::*;

const NUM_RECEIVERS: usize = 7;
const ITERS: usize = 50;
const WAIT_MS: u64 = 200;

fn main() {
    let mut sender: Sender<Instant> = Sender::new();
    let mut threads = Vec::new();
    let gtimes = Arc::new(Mutex::new(Vec::<Duration>::new()));
    for _ in 0..NUM_RECEIVERS {
        let mut receiver = sender.receiver();
        let gtimes = gtimes.clone();
        let mut ltimes = Vec::with_capacity(ITERS);
        threads.push(thread::spawn(move || loop {
            match receiver.recv() {
                Ok(x) => {
                    let dur = x.elapsed();
                    ltimes.push(dur);
                    thread::sleep(Duration::from_millis(1));
                }
                Err(RecvError::Orphaned) => {
                    gtimes.lock().unwrap().extend(&ltimes);
                    break;
                }
            }
        }));
    }

    let mut pb = pbr::ProgressBar::new(ITERS as u64);
    thread::sleep(Duration::from_millis(100));
    for _ in 0..ITERS {
        pb.inc();
        let now = Instant::now();
        loop {
            if sender.send(Box::new(now)).is_some() { break }
        }
        sender.unblock();
        thread::sleep(Duration::from_millis(WAIT_MS));
    }

    ::std::mem::drop(sender);
    for t in threads { t.join().unwrap(); }
    println!("{}", mk_stats(&gtimes.lock().unwrap()));
}
