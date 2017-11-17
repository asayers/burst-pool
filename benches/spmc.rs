extern crate pbr;
extern crate spmc;

mod settings; use settings::*;

use std::io::stderr;
use std::thread;
use std::time::*;

fn main() {
    for spec in BENCH_SPECS.iter() {
        println!("# {:?}\n\"spmc {}/{}\"", spec, spec.num_msgs, spec.num_receivers);
        bench(spec);
        println!("\n\n");
    }
}

fn bench(spec: &BenchSpec) {
    let (sender, receiver) = spmc::channel();

    // let gtimes = Arc::new(Mutex::new(Vec::<Duration>::new()));
    let mut threads = Vec::new();
    for _ in 0..spec.num_receivers {
        let receiver: spmc::Receiver<Instant> = receiver.clone();
        // let gtimes = gtimes.clone();
        let mut ltimes = Vec::with_capacity(spec.iters);
        let lines = spec.iters * spec.num_msgs;
        let gaussian_weight = (lines as f64).recip();
        threads.push(thread::spawn(move|| loop {
            match receiver.recv() {
                Ok(x) => {
                    let dur = x.elapsed();
                    ltimes.push(dur);
                    thread::sleep(Duration::from_millis(1));
                }
                Err(_) => {
                    // gtimes.lock().unwrap().extend(&ltimes);
                    for time in ltimes {
                        println!("{}{:09} {}", time.as_secs(), time.subsec_nanos(), gaussian_weight);
                    }
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
            sender.send(now).unwrap();
        }
        thread::sleep(Duration::from_millis(spec.wait_ms));
    }

    ::std::mem::drop(sender);
    for t in threads { t.join().unwrap(); }
    // let times = gtimes.lock().unwrap();
    // Stats::new(&times)
}
