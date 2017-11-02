extern crate burst_pool;
extern crate pbr;

mod stats;    use stats::*;
mod settings; use settings::*;

use burst_pool::*;
use std::io::stderr;
use std::sync::*;
use std::thread;
use std::time::*;

fn main() {
    println!("burst_chan");
    for spec in BENCH_SPECS.iter() {
        println!("{:?}\n{}", spec, bench(spec));
    }
}

fn bench(spec: &BenchSpec) -> Stats {
    let mut sender: Sender<Instant> = Sender::new();
    let mut threads = Vec::new();
    let gtimes = Arc::new(Mutex::new(Vec::<Duration>::new()));
    for _ in 0..spec.num_receivers {
        let mut receiver = sender.mk_receiver();
        let gtimes = gtimes.clone();
        let mut ltimes = Vec::with_capacity(spec.iters);
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

    let mut pb = pbr::ProgressBar::on(stderr(), spec.iters as u64);
    thread::sleep(Duration::from_millis(100));
    for _ in 0..spec.iters {
        pb.inc();
        let now = Instant::now();
        for _ in 0..spec.num_msgs {
            if sender.send(Box::new(now)).is_some() { break }
        }
        sender.wake_all();
        thread::sleep(Duration::from_millis(spec.wait_ms));
    }

    ::std::mem::drop(sender);
    for t in threads { t.join().unwrap(); }
    let times = gtimes.lock().unwrap();
    Stats::new(&times)
}
