use histogram::Histogram;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool,Ordering};
use std::thread;
use std::time::*;

const THREADS: usize = 10;

lazy_static!{
    static ref HIST: Mutex<(Instant, Histogram)> = Mutex::new((Instant::now(), Histogram::new()));
    static ref FLAG: AtomicBool = AtomicBool::new(false);
}

pub fn bench() -> Histogram {
    let mut handles = vec![];
    info!("Spawning workers...");
    for _ in 0..THREADS {
        handles.push(thread::spawn(|| worker()));
    }

    for _ in 0..500 {
        thread::sleep(Duration::from_millis(10));
        {
            let mut x = HIST.lock().unwrap();
            x.0 = Instant::now();
        }
        for i in 0..THREADS {
            handles[i].thread().unpark();
        }
    }
    thread::sleep(Duration::from_millis(10));
    FLAG.store(true, Ordering::Release);
    for h in handles.into_iter() {
        h.thread().unpark();
        h.join().unwrap();
    }
    HIST.lock().unwrap().1.clone()
}

fn worker() {
    info!("Started worker");
    loop {
        thread::park();
        let ts = Instant::now();
        thread::sleep(Duration::from_millis(2));
        if FLAG.load(Ordering::Acquire) == false {
            let mut x = HIST.lock().unwrap();
            let micros = ts.duration_since(x.0).subsec_nanos() as f64 / 1_000.0;
            x.1.add(micros);
        } else {
            break
        }
    }
    info!("Stopped worker");
}
