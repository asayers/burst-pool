extern crate burst_pool;
#[macro_use] extern crate lazy_static;
extern crate net2;

use burst_pool::BurstPool;
use net2::UdpBuilder;
use std::net::UdpSocket;
use std::sync::Mutex;
use std::thread;
use std::time::*;

mod histogram; use histogram::Histogram;

lazy_static!{
    static ref HIST: Mutex<Histogram> = Mutex::new(Histogram::new());
}

fn bind_sock() -> UdpSocket {
    || -> ::std::io::Result<UdpSocket> {
        let sock = UdpBuilder::new_v4()?.reuse_address(true)?.bind(("0.0.0.0", 14360))?;
        sock.connect("127.0.0.1:12345")?;
        Ok(sock)
    }().expect("Failed to bind sock")
}

#[test] #[ignore]
fn bench_pool_socket() {
    let n = 10;
    HIST.lock().unwrap().clear();
    let mut pool = BurstPool::new();
    for i in 0..n {
        let sock = bind_sock();
        pool.spawn(move|recv_ts: Instant| {
            let micros = recv_ts.elapsed().subsec_nanos() as f64 / 1_000.0;
            let msg = format!("[thread {}] {:>5.0?} us\n", i, micros);
            sock.send(msg.as_bytes()).expect("Couldn't send message");
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

// Benchmarking an alternative implementation for sending many values at once.
fn send_many<T>(pool: &mut BurstPool<T>, xs: Vec<T>) {
    let mut i = 0;
    let n = xs.len();
    for x in xs {
        let idx = (pool.next_thread + i) % pool.threads.len();
        let (_, ref mut tx) = pool.threads[idx];
        tx.send(x).unwrap();
        i += 1;
    }
    for i in 0..n {
        let idx = (pool.next_thread + i) % pool.threads.len();
        let (ref mut handle, _) = pool.threads[idx];
        handle.thread().unpark();
    }
    pool.next_thread = (pool.next_thread + n) % pool.threads.len();
}

#[test] #[ignore]
fn bench_pool2() {
    let n = 10;
    HIST.lock().unwrap().clear();
    let mut pool = BurstPool::new();
    for i in 0..n {
        let sock = bind_sock();
        pool.spawn(move|recv_ts: Instant| {
            let micros = recv_ts.elapsed().subsec_nanos() as f64 / 1_000.0;
            let msg = format!("[thread {}] {:>5.0?} us\n", i, micros);
            sock.send(msg.as_bytes()).expect("Couldn't send message");
            thread::sleep(Duration::from_millis(10));
            HIST.lock().unwrap().add(micros);
        });
    }
    for _ in 0..100 {
        thread::sleep(Duration::from_millis(100));
        let now = Instant::now();
        send_many(&mut pool, vec![now;10]);
    }
    thread::sleep(Duration::from_millis(100));
    println!("{}", *HIST.lock().unwrap());
}

#[test] #[ignore]
fn bench_pool3() {
    let n = 10;
    HIST.lock().unwrap().clear();
    let mut pool = BurstPool::new();
    for i in 0..(n-1) {
        let sock = bind_sock();
        pool.spawn(move|recv_ts: Instant| {
            let micros = recv_ts.elapsed().subsec_nanos() as f64 / 1_000.0;
            let msg = format!("[thread {}] {:>5.0?} us\n", i, micros);
            sock.send(msg.as_bytes()).expect("Couldn't send message");
            thread::sleep(Duration::from_millis(10));
            HIST.lock().unwrap().add(micros);
        });
    }
    let sock = bind_sock();
    for _ in 0..100 {
        thread::sleep(Duration::from_millis(100));
        let now = Instant::now();
        send_many(&mut pool, vec![now;9]);
        let micros = now.elapsed().subsec_nanos() as f64 / 1_000.0;
        let msg = format!("[thread {}] {:>5.0?} us\n", n-1, micros);
        sock.send(msg.as_bytes()).expect("Couldn't send message");
        thread::sleep(Duration::from_millis(10));
        HIST.lock().unwrap().add(micros);
    }
    thread::sleep(Duration::from_millis(100));
    println!("{}", *HIST.lock().unwrap());
}
