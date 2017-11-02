extern crate pbr;
extern crate spmc;

mod stats;    use stats::*;

use std::io::stderr;
use std::sync::*;
use std::thread;
use std::time::*;

const NUM_RECEIVERS: usize = 7;
const ITERS: usize = 50;
const WAIT_MS: u64 = 200;

fn main() {
    let (sender, receiver) = spmc::channel();

    let gtimes = Arc::new(Mutex::new(Vec::<Duration>::new()));
    let mut threads = Vec::new();
    for _ in 0..NUM_RECEIVERS {
        let receiver: spmc::Receiver<Instant> = receiver.clone();
        let gtimes = gtimes.clone();
        let mut ltimes = Vec::with_capacity(ITERS);
        threads.push(thread::spawn(move|| loop {
            match receiver.recv() {
                Ok(x) => {
                    let dur = x.elapsed();
                    ltimes.push(dur);
                    thread::sleep(Duration::from_millis(1));
                }
                Err(_) => {
                    gtimes.lock().unwrap().extend(&ltimes);
                    break;
                }
            }
        }));
    }

    let mut pb = pbr::ProgressBar::on(stderr(), spec.iters as u64);
    thread::sleep(Duration::from_millis(100));
    for _ in 0..ITERS {
        pb.inc();
        let now = Instant::now();
        for _ in 0..NUM_RECEIVERS {
            sender.send(now).unwrap();
        }
        thread::sleep(Duration::from_millis(WAIT_MS));
    }

    ::std::mem::drop(sender);
    for t in threads { t.join().unwrap(); }
    println!("{}", mk_stats(&gtimes.lock().unwrap()));
}
