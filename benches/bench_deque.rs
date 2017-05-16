use histogram::Histogram;
use deque;
use std::sync::Mutex;
use std::thread;
use std::time::*;

const THREADS: usize = 10;

lazy_static!{
    static ref HIST: Mutex<Histogram> = Mutex::new(Histogram::new());
}

pub fn bench() -> Histogram {
    HIST.lock().unwrap().clear();
    let (tx, rx) = deque::new();

    let mut handles = Vec::new();
    for _ in 0..THREADS {
        let rx: deque::Stealer<Instant> = rx.clone();
        handles.push(thread::spawn(move|| {
            loop {
                use deque::Stolen::*;
                match rx.steal() {
                    Data(ts) => {
                        let micros = ts.elapsed().subsec_nanos() as f64 / 1_000.0;
                        thread::sleep(Duration::from_millis(2));
                        HIST.lock().unwrap().add(micros);
                    },
                    Empty => thread::park(),
                    Abort => break,
                }
            }
        }));
    }

    for _ in 0..500 {
        thread::sleep(Duration::from_millis(10));
        let now = Instant::now();
        for _ in 0..THREADS { tx.push(now); }
        for h in handles.iter() { h.thread().unpark(); }
    }

    thread::sleep(Duration::from_millis(10));
    for h in handles.iter() { h.thread().unpark(); }
    HIST.lock().unwrap().clone()
}
