extern crate burst_pool;
extern crate pbr;
use std::thread;
use std::time::*;
use burst_pool::*;

const NUM_RECEIVERS: usize = 7;
const ITERS: usize = 250;

fn main() {
    let mut sender: Sender<Instant> = Sender::new();
    let mut threads = Vec::new();
    for _ in 0..NUM_RECEIVERS {
        let mut receiver = sender.receiver();
        let mut times = Vec::with_capacity(ITERS);
        threads.push(thread::spawn(move || loop {
            match receiver.recv() {
                Ok(x) => {
                    let dur = x.elapsed();
                    times.push(dur);
                    thread::sleep(Duration::from_millis(1));
                }
                Err(RecError::Orphaned) => {
                    times.sort();
                    let avg = times.iter().map(Duration::subsec_nanos).sum::<u32>() /
                                times.len() as u32;
                    let med = times[times.len() / 2].subsec_nanos();
                    let best = times[0].subsec_nanos();
                    let worst = times[times.len() - 1].subsec_nanos();
                    println!("{} loops => avg {:>5} ns, med {:>5} ns, range {}..{:<7}  - {:?}", times.len(), avg, med, best, worst, thread::current().id());
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
        thread::sleep(Duration::from_millis(10));
    }

    ::std::mem::drop(sender);
    for t in threads { t.join().unwrap(); }
}
