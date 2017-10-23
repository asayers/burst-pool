use histogram::Histogram;
use std::sync::Mutex;
use std::thread;
use std::time::*;
use threadpool::ThreadPool;

const THREADS: usize = 10;

lazy_static!{
    static ref HIST: Mutex<Histogram> = Mutex::new(Histogram::new());
}

pub fn bench() -> Histogram {
    HIST.lock().unwrap().clear();
    let pool = ThreadPool::new(THREADS);

    for _ in 0..500 {
        thread::sleep(Duration::from_millis(10));
        let now = Instant::now();
		for _ in 0..10 {
			pool.execute(move|| {
				let micros = now.elapsed().subsec_nanos() as f64 / 1_000.0;
				thread::sleep(Duration::from_millis(2));
				HIST.lock().unwrap().add(micros);
			});
		}
    }

    thread::sleep(Duration::from_millis(10));
    HIST.lock().unwrap().clone()
}
