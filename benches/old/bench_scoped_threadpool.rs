use histogram::Histogram;
use scoped_threadpool::Pool;
use std::sync::Mutex;
use std::thread;
use std::time::*;

const THREADS: u32 = 10;

lazy_static!{
    static ref HIST: Mutex<Histogram> = Mutex::new(Histogram::new());
}

pub fn bench() -> Histogram {
    HIST.lock().unwrap().clear();
    let mut pool = Pool::new(THREADS);

    for _ in 0..500 {
        thread::sleep(Duration::from_millis(10));
        let now = Instant::now();
        pool.scoped(|scope| {
            for _ in 0..10 {
                scope.execute(move|| {
                    let micros = now.elapsed().subsec_nanos() as f64 / 1_000.0;
                    thread::sleep(Duration::from_millis(2));
                    HIST.lock().unwrap().add(micros);
                });
            }
        });
    }

    thread::sleep(Duration::from_millis(10));
    HIST.lock().unwrap().clone()
}
