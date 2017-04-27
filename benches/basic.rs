use burst_pool::BurstPool;
use histogram::Histogram;
use std::sync::Mutex;
use std::thread;
use std::time::*;

const THREADS: usize = 10;

lazy_static!{
    static ref HIST: Mutex<Histogram> = Mutex::new(Histogram::new());
}

pub fn bench() -> Histogram {
    HIST.lock().unwrap().clear();
    let mut pool = BurstPool::new();
    for _ in 0..THREADS {
        pool.spawn(move|recv_ts: Instant| {
            let micros = recv_ts.elapsed().subsec_nanos() as f64 / 1_000.0;
            // let _ = format!("[thread {}] {:>5.0?} us\n", i, micros);
            thread::sleep(Duration::from_millis(2));
            HIST.lock().unwrap().add(micros);
        });
    }
    for _ in 0..500 {
        thread::sleep(Duration::from_millis(10));
        let now = Instant::now();
        for _ in 0..THREADS { pool.send(now); }
    }
    thread::sleep(Duration::from_millis(10));
    HIST.lock().unwrap().clone()
}
