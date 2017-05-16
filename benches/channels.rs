#[cfg(nightly)] extern crate bounded_spsc_queue;
extern crate burst_pool;
#[macro_use] extern crate lazy_static;
extern crate lossyq;
extern crate net2;
extern crate rb;
extern crate spmc;

#[cfg(nightly)] use bounded_spsc_queue as spsc;
use net2::UdpBuilder;
use rb::{RB,RbProducer,RbConsumer};
use std::net::UdpSocket;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration,Instant};

mod histogram; use histogram::Histogram;

lazy_static!{
    static ref HIST: Mutex<Histogram> = Mutex::new(Histogram::new());
}

fn _bind_sock() -> UdpSocket {
    || -> ::std::io::Result<UdpSocket> {
        let sock = UdpBuilder::new_v4()?.reuse_address(true)?.bind(("0.0.0.0", 14360))?;
        sock.connect("127.0.0.1:12345")?;
        Ok(sock)
    }().expect("Failed to bind sock")
}

#[test] #[cfg(nightly)] #[ignore]
fn bench_spsc() {
    let n = 10;
    HIST.lock().unwrap().clear();
    for round in 0..100 {
        let mut threads = Vec::new();
        for i in 0..n {
            let (tx,rx) = spsc::make(5);
            threads.push((tx,thread::spawn(move|| {
                // let sock = bind_sock();
                thread::park();
                let recv_ts: Instant = rx.pop();  // block
                let micros = recv_ts.elapsed().subsec_nanos() as f64 / 1_000.0;
                let _ = format!("[round {}; thread {}] {:>5.0?} us\n", round, i, micros);
                // sock.send(msg.as_bytes()).expect("Couldn't send message");
                thread::sleep(Duration::from_millis(10));
                HIST.lock().unwrap().add(micros);
            })));
        }
        thread::sleep(Duration::from_millis(100));
        let now = Instant::now();
        for &mut (ref mut tx, ref mut thread) in threads.iter_mut() { tx.push(now); thread.thread().unpark(); }
        for (_, thread) in threads.drain(..) {
            thread.join().unwrap();
        }
    }
    println!("{}", *HIST.lock().unwrap());
}

#[test] #[ignore]
fn bench_lossyq() {
    let n = 10;
    HIST.lock().unwrap().clear();
    for round in 0..100 {
        let mut threads = Vec::new();
        for i in 0..n {
            let (tx,mut rx) = lossyq::spsc::channel(5);
            threads.push((tx,thread::spawn(move|| {
                // let sock = bind_sock();
                thread::park();
                let recv_ts: Instant = rx.iter().next().unwrap();
                let micros = recv_ts.elapsed().subsec_nanos() as f64 / 1_000.0;
                let _ = format!("[round {}; thread {}] {:>5.0?} us\n", round, i, micros);
                // sock.send(msg.as_bytes()).expect("Couldn't send message");
                thread::sleep(Duration::from_millis(10));
                HIST.lock().unwrap().add(micros);
            })));
        }
        thread::sleep(Duration::from_millis(100));
        let now = Instant::now();
        for &mut (ref mut tx, ref mut thread) in threads.iter_mut() { tx.put(|x| *x = Some(now)); thread.thread().unpark(); }
        for (_, thread) in threads.drain(..) {
            thread.join().unwrap();
        }
    }
    println!("{}", *HIST.lock().unwrap());
}

#[test] #[ignore]
fn bench_rb() {
    let n = 10;
    HIST.lock().unwrap().clear();
    for round in 0..100 {
        let mut threads = Vec::new();
        for i in 0..n {
            let rb = rb::SpscRb::new(5);
            let rx = rb.consumer();
            let tx = rb.producer();
            threads.push((tx,thread::spawn(move|| {
                // let sock = bind_sock();
                thread::park();
                let mut foo: [Option<Instant>;1] = [None;1];
                rx.read_blocking(&mut foo).unwrap();
                let recv_ts: Instant = foo[0].unwrap();
                let micros = recv_ts.elapsed().subsec_nanos() as f64 / 1_000.0;
                let _ = format!("[round {}; thread {}] {:>5.0?} us\n", round, i, micros);
                // sock.send(msg.as_bytes()).expect("Couldn't send message");
                thread::sleep(Duration::from_millis(10));
                HIST.lock().unwrap().add(micros);
            })));
        }
        thread::sleep(Duration::from_millis(100));
        let now = Instant::now();
        let data = [Some(now)];
        for &mut (ref mut tx, ref mut thread) in threads.iter_mut() { tx.write(&data).unwrap(); thread.thread().unpark(); }
        for (_, thread) in threads.drain(..) {
            thread.join().unwrap();
        }
    }
    println!("{}", *HIST.lock().unwrap());
}
