extern crate burst_pool;
#[macro_use] extern crate lazy_static;

use burst_pool::BurstPool;
use std::sync::Mutex;
use std::thread;
use std::time::*;

mod histogram; use histogram::Histogram;

lazy_static!{
    static ref HIST: Mutex<Histogram> = Mutex::new(Histogram::new());
}

#[test]
fn bench_pool() {
    let n = 10;   // 10 threads
    HIST.lock().unwrap().clear();
    let mut pool = BurstPool::new();
    for i in 0..n {
        pool.spawn(move|recv_ts: Instant| {
            let micros = recv_ts.elapsed().subsec_nanos() as f64 / 1_000.0;
            let _ = format!("[thread {}] {:>5.0?} us\n", i, micros);
            thread::sleep(Duration::from_millis(10));
            HIST.lock().unwrap().add(micros);
        });
    }
    for _ in 0..100 {
        thread::sleep(Duration::from_millis(100));
        let now = Instant::now();
        for _ in 0..n { pool.send(now); }
    }
    thread::sleep(Duration::from_millis(100));
    println!("{}", *HIST.lock().unwrap());
}
