use histogram::Histogram;
use std::sync::Mutex;
use std::thread;
use std::time::*;
use jobsteal::make_pool;

const THREADS: usize = 10;

lazy_static!{
    static ref HIST: Mutex<Histogram> = Mutex::new(Histogram::new());
}

pub fn bench() -> Histogram {
    HIST.lock().unwrap().clear();
    let mut pool = make_pool(THREADS).unwrap();

    for _ in 0..500 {
        thread::sleep(Duration::from_millis(10));
        let now = Instant::now();
        for _ in 0..10 {
            pool.submit(move|| {
                let micros = now.elapsed().subsec_nanos() as f64 / 1_000.0;
                thread::sleep(Duration::from_millis(2));
                HIST.lock().unwrap().add(micros);
            });
        }
    }

    thread::sleep(Duration::from_millis(10));
    HIST.lock().unwrap().clone()
}
