use histogram::Histogram;
use spmc;
use std::sync::Mutex;
use std::thread;
use std::time::*;

const THREADS: usize = 10;

lazy_static!{
    static ref HIST: Mutex<Histogram> = Mutex::new(Histogram::new());
}

pub fn bench() -> Histogram {
    HIST.lock().unwrap().clear();
    let (tx, rx) = spmc::channel();

    let mut handles = Vec::new();
    for _ in 0..THREADS {
        let rx: spmc::Receiver<Instant> = rx.clone();
        handles.push(thread::spawn(move|| {
            while let Ok(ts) = rx.recv() {
                let micros = ts.elapsed().subsec_nanos() as f64 / 1_000.0;
                thread::sleep(Duration::from_millis(2));
                HIST.lock().unwrap().add(micros);
            }
        }));
    }

    for _ in 0..500 {
        thread::sleep(Duration::from_millis(10));
        let now = Instant::now();
        for _ in 0..THREADS { tx.send(now).unwrap(); }
    }

    thread::sleep(Duration::from_millis(10));
    HIST.lock().unwrap().clone()
}
