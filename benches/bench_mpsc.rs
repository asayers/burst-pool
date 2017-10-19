use histogram::Histogram;
use std::sync::Mutex;
use std::thread;
use std::time::*;
use std::sync::mpsc;

const THREADS: usize = 10;

lazy_static!{
    static ref HIST: Mutex<Histogram> = Mutex::new(Histogram::new());
}

pub fn bench() -> Histogram {
    HIST.lock().unwrap().clear();

    let mut threads = Vec::new();
    let mut next_thread = 0;

    for _ in 0..THREADS {
        let (tx, rx) = mpsc::channel::<Instant>();
        threads.push(tx);
        thread::spawn(move|| {
            loop {
                match rx.recv() {
                    Ok(x) => {
                        let micros = x.elapsed().subsec_nanos() as f64 / 1_000.0;
                        // let _ = format!("[thread {}] {:>5.0?} us\n", i, micros);
                        thread::sleep(Duration::from_millis(2));
                        HIST.lock().unwrap().add(micros);
                    }
                    Err(_) => break,
                }
            }
        });
    }
    for _ in 0..500 {
        thread::sleep(Duration::from_millis(10));
        let now = Instant::now();
        for _ in 0..THREADS {
            next_thread = next_thread % threads.len();
            threads[next_thread].send(now).unwrap();
            next_thread += 1;
        }
    }
    thread::sleep(Duration::from_millis(10));
    HIST.lock().unwrap().clone()
}
