use histogram::Histogram;
use std::thread;
use std::time::*;

const THREADS: usize = 10;

pub fn bench() -> Histogram {
    let mut hist = Histogram::new();

    for _ in 0..500 {
        let mut handles = vec![];
        for _ in 0..THREADS {
            handles.push(thread::spawn(|| {
                thread::park();
            }));
        }
        thread::sleep(Duration::from_millis(10));
        let start = Instant::now();
        for i in 0..THREADS {
            handles[i].thread().unpark();
        }
        for handle in handles {
            handle.join().unwrap();
        }
        let micros = start.elapsed().subsec_nanos() as f64 / 1_000.0;
        hist.add(micros);
    }
    thread::sleep(Duration::from_millis(10));
    hist
}
