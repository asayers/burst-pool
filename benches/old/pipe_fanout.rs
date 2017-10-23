use histogram::Histogram;
use mio::*;
use mio::unix::EventedFd;
use nix::unistd::{pipe,write};
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

/// This one works by using a pipe per thread and edge-triggered epoll.
pub fn bench() {
    let mut handles = vec![];
    info!("Spawning workers...");
    for _ in 0..THREADS {
        let (pipe_rx, pipe_tx) = pipe().expect("couldn't make pipe");
        handles.push((pipe_tx, thread::spawn(move|| worker(pipe_rx).expect("thread died"))));
    }

    for _ in 0..100 {
        thread::sleep(Duration::from_millis(10));
        {
            let mut x = HIST.lock().unwrap();
            x.0 = Instant::now();
        }
        for i in 0..THREADS {
            write(handles[i as usize].0, &[0]).expect("couldn't write to pipe");
        }
    }

    thread::sleep(Duration::from_millis(10));
    println!("{}", (*HIST.lock().unwrap()).1);

    (*FLAG).store(true, Ordering::Release);
    for (pipe, h) in handles.into_iter() {
        write(pipe, &[0]).expect("couldn't write to pipe");
        h.join().unwrap();
    }
}

fn worker(pipe_rx: RawFd) -> Result<(),io::Error> {
    let poll = Poll::new().unwrap();
    poll.register(&EventedFd(&pipe_rx), Token(0), Ready::readable(), PollOpt::edge())?;
    let mut events = Events::with_capacity(16);
    info!("Started worker");
    loop {
        poll.poll(&mut events, None)?;
        let ts = Instant::now();
        thread::sleep(Duration::from_millis(50));
        if (*FLAG).load(Ordering::Acquire) == false {
            let mut x = HIST.lock().unwrap();
            let micros = ts.duration_since(x.0).subsec_nanos() as f64 / 1_000.0;
            x.1.add(micros);
        } else {
            break
        }
    }
    Ok(())
}
