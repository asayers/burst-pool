use histogram::Histogram;
use mio::*;
use mio::unix::EventedFd;
use nix::unistd::{pipe,write,read};
use rand;
use std::io;
use std::os::unix::io::RawFd;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool,Ordering};
use std::thread;
use std::time::*;

const THREADS: u8 = 10;

lazy_static!{
    static ref HIST: Mutex<(Instant, Histogram)> = Mutex::new((Instant::now(), Histogram::new()));
    static ref FLAG: AtomicBool = AtomicBool::new(false);
}

/// This one works by using a single pipe end edge-triggered epoll.
pub fn bench() -> Histogram {
    let (pipe_rx, pipe_tx) = pipe().expect("couldn't make pipe");
    let mut handles = vec![];
    info!("Spawning workers...");
    for _ in 0..THREADS {
        handles.push(thread::spawn(move|| worker(pipe_rx, pipe_tx).expect("thread died")));
    }

    for _ in 0..250 {
        thread::sleep(Duration::from_millis(20));
        let n: u8 = (rand::random::<u8>() % (THREADS - 1)) + 1;
        info!("Waking {} threads...", n);
        {
            let mut x = HIST.lock().unwrap();
            x.0 = Instant::now();
        }
        write(pipe_tx, &[n]).expect("couldn't write to pipe");
    }
    thread::sleep(Duration::from_millis(10));
    FLAG.store(true, Ordering::Release);
    write(pipe_tx, &[THREADS]).expect("couldn't write to pipe");
    for h in handles.into_iter() {
        h.join().unwrap();
    }
    HIST.lock().unwrap().1.clone()
}

fn worker(pipe_rx: RawFd, pipe_tx: RawFd) -> Result<(),io::Error> {
    let poll = Poll::new().unwrap();
    poll.register(&EventedFd(&pipe_rx), Token(0), Ready::readable(), PollOpt::edge())?;
    let mut events = Events::with_capacity(16);
    let mut buf = [0u8];
    info!("Started worker");
    for _ in 0..100 {
        poll.poll(&mut events, None)?;
        let n = read(pipe_rx, &mut buf).expect("read failed");
        assert!(n == 1);
        let remaining = buf[0];
        let ts = Instant::now();
        if remaining > 1 {
            write(pipe_tx, &[remaining-1]).expect("Couldn't write to pipe");
        }
        thread::sleep(Duration::from_millis(2));
        if FLAG.load(Ordering::Acquire) == false {
            let mut x = HIST.lock().unwrap();
            let micros = ts.duration_since(x.0).subsec_nanos() as f64 / 1_000.0;
            x.1.add(micros);
        } else {
            break
        }
    }
    Ok(())
}
