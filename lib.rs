/*! A thread pool optimised for bursts of activity.

Consider the following use-case: A single thread produces work which must then be performed by a
pool of worker threads. The work is produced infrequently, but in bursts. Under normal operation,
therefore, the threads in the pool sleep until some event requires many of them to be suddenly be
woken at once. Those threads perform some work before going back to sleep again.

Most thread pools schedule work based on thread readiness. This invariably means that work is
pushed onto a single queue which is shared by all the workers, and the workers steal work from the
queue when they're ready to accept it. This usually works very well, but not in our use-case: since
many threads become runnable at the same time, they immediately contend to read from the queue.

Instead, we use a round-robin scheduling strategy, in which the work-producing thread sends work to
specific workers. This eliminates contention (since each worker has its own queue). The trade-off
is that it performs badly when utilisation is high and the workloads are uneven.

# Usage

Normally a thread pool will spawn a fixed number of worker threads; once the threads are running,
you send closures to the pool which are then executed by one of the workers. `BurstPool`'s API
works a bit differently.

A `BurstPool` is parameterised over the kind of data which will be sent to the workers. (This data
could be boxed closures, if you want to mimic the normal API.) The user provides work by calling
`BurstPool::send()`, and one of the workers is chosen to receive it. Each worker has its own
function which it uses to handle work sent to it. This function is provided when the thread is
spawned.

```
use burst_pool::BurstPool;

let mut pool = BurstPool::new();

// Spawn a worker in the pool
pool.spawn(|x| println!("Received {}!", x));

// Send some work to the worker
pool.send(36).unwrap();
```

# Performance

I'm using the following benchmark:

1. Create a pool with 10 workers.
2. Send 10 pieces of work to the pool.
3. When a thread recieves some work it reads the clock, records its latency, and goes back to
   sleep.

For comparison, the benchmark was replicated using some of the other thread pools on crates.io
(which are not optimised for this use-case), and I also benchmarked the time taken to suddenly
unpark 10 parked threads.

<img src="https://raw.githubusercontent.com/asayers/burst-pool/master/histogram.png" style="margin: 0 auto; display: block;" />

crate               | mean latency | 20 %ⁱˡᵉ | 40 %ⁱˡᵉ  | 60 %ⁱˡᵉ  | 80 %ⁱˡᵉ
--------------------|--------------|---------|----------|----------|-----------
[burst_pool]        | 8.3 μs       | <3.4 μs | <5.1 μs  | <7.6 μs  | <7.6 μs
[threadpool]        | 17.4 μs      | <7.6 μs | <11.4 μs | <17.1 μs | <17.1 μs
[scoped_threadpool] | 18.7 μs      | <7.6 μs | <11.4 μs | <17.1 μs | <17.1 μs
(unpark only)       | 7.7 μs       | <3.4 μs | <5.1 μs  | <7.6 μs  | <7.6 μs

[burst_pool]: https://docs.rs/burst_pool/
[threadpool]: https://docs.rs/threadpool/
[scoped_threadpool]: https://docs.rs/scoped_threadpool/

- The profile of `BurstPool` shows that it doesn't add much latency over just calling `unpark()`.
  Almost all of the time goes to the linux scheduler.
- The mean latency is heavily skewed by outliers, lying above the 80th percentile. You can expect
  latencies better than 7.6 μs most of the time, with the occasional long wait.
- Getting results as good as these is heavily dependent on setting `max_cstate = 0` - this makes a
  huge difference to thread wake-up times.

You can run the benchmarks yourself with `cargo bench`. If your results are significantly worse
than those above, your kernel might be powering down CPU cores too eagerly. If you care about
latency more than battery life, consider setting `max_cstate = 0`.
*/

#[cfg(test)] extern crate env_logger;
#[macro_use] extern crate log;

use std::error::Error;
use std::fmt::{self, Debug, Display};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};

/// A thread pool optimised for bursts of activity.
///
/// A `BurstPool` maintains one unbounded [channel] for each thread. Because these channels have
/// only one producer, the more efficient spsc implementation will be used.
///
/// [channel]: https://doc.rust-lang.org/std/sync/mpsc/fn.channel.html
///
/// Threads spawned in the pool are parked (they do *not* busy-wait) until there is work to be
/// done. When `BurstPool::send()` is called, a thread is chosen, the payload is pushed onto its
/// queue, and it is unparked. Threads are selected round-robin, so uneven processing time is not
/// handled well.
///
/// When the `BurstPool` is dropped, all threads will be signalled to exit. Dropping will then
/// block until the threads are all dead.
///
/// If a thread panics it will go undetected until we attempt to send some work to it. At this
/// point, the thread will be removed from the pool and a warning will be issued, and the work will
/// be sent to the next thread in the ring. Dead threads are not replenished.
///
/// ## Example
///
/// ```
/// use burst_pool::BurstPool;
///
/// let mut pool = BurstPool::new();
///
/// for thread_id in 0..3 {
///     pool.spawn(move|x| {
///         println!("Thread {} received {}", thread_id, x);
///     });
/// }
///
/// for x in 0..5 {
///     pool.send(x).unwrap();
/// }
/// ```
///
/// This will print something like:
///
/// ```none
/// Thread 0 received 0
/// Thread 0 received 3
/// Thread 1 received 1
/// Thread 1 received 4
/// Thread 2 received 2
/// ```
///
pub struct BurstPool<T> {
    threads: Vec<(JoinHandle<()>, mpsc::Sender<T>)>,
    next_thread: usize,
}

impl<T> BurstPool<T> where T: Send {
    /// Create a new empty pool.
    pub fn new() -> BurstPool<T> {
        BurstPool {
            threads: Vec::new(),
            next_thread: 0,
        }
    }

    /// Spawn a thread in the pool.
    ///
    /// The spawned thread blocks until `pool.send()` is called, at which point it may be woken up.
    /// When woken, it runs the given closure with the value it recieved, before going back to
    /// sleep.
    ///
    /// Since you typically don't know which thread a `send()` call will wake up, all workers in
    /// a pool should do pretty similar things with the work they receive. Also, bear in mind that
    /// the `BurstPool`'s scheduling strategy doesn't cope well with uneven processing time -
    /// particularly when one thread is systematically slower than the others.
    pub fn spawn<F>(&mut self, mut consume: F)
            where F: FnMut(T) + Send + 'static, T: 'static {
        let (tx,rx) = mpsc::channel();
        let handle = thread::spawn(move|| {
            loop {
                use self::mpsc::TryRecvError::*;
                match rx.try_recv() {
                    Ok(x) => consume(x),
                    Err(Empty) => thread::park(),  // May unblock spuriously.
                    Err(Disconnected) => break,    // Kill the thread
                }
            }
        });
        self.threads.push((handle, tx));
    }

    /// Send a value to be processed by one of the threads in the pool.
    ///
    /// If there are no threads available to perform the work, an error is returned containing the
    /// unperformed work.
    pub fn send(&mut self, x: T) -> Result<(), BurstError<T>> {
        if self.threads.is_empty() { return Err(BurstError::NoThreads(x)); }
        self.next_thread = self.next_thread % self.threads.len();
        let ret = self.threads[self.next_thread].1.send(x);
        match ret {
            Ok(()) => {
                // Data sent successfully: unpark and advance next_thread.
                self.threads[self.next_thread].0.thread().unpark();
                self.next_thread += 1;
                Ok(())
            },
            Err(mpsc::SendError(x)) => {
                // The thread we tried to send to is dead: remove it from the list.
                // FIXME: We don't seem to reliably take this branch when a worker panics. In such
                // cases we silently drop work!
                let (handle, _) = self.threads.remove(self.next_thread);
                match handle.join() {
                    Ok(()) => unreachable!(),
                    Err(_) => error!("Worker thread panicked! Removing from pool and retrying..."),
                }
                self.send(x)
            },
        }
    }
}

impl<T> Drop for BurstPool<T> {
    /// Signal all the threads in the pool to shut down, and wait for them to do so.
    fn drop(&mut self) {
        for (handle, tx) in self.threads.drain(..) {
            ::std::mem::drop(tx);
            handle.thread().unpark();
            handle.join().unwrap_or_else(|_| error!("Worker thread panicked! Never mind..."));
        }
    }
}

#[derive(Debug)]
pub enum BurstError<T> {
    NoThreads(T),
}

impl<T> Display for BurstError<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BurstError::NoThreads(_) => write!(f, "No threads in pool"),
        }
    }
}

impl<T> Error for BurstError<T> where T: Debug + Display {
    fn description(&self) -> &str {
        match *self {
            BurstError::NoThreads(_) => "No threads in pool",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panic_1() {
        // env_logger::init().unwrap();
        let mut pool = BurstPool::new();
        pool.spawn(|_| panic!("whoa!"));
        pool.spawn(|x| println!("got {}", x));
        println!("still ok");
        pool.send(1).unwrap();
        println!("still ok");
    }

    #[test]
    fn test_panic_2() {
        // env_logger::init().unwrap();
        let mut pool = BurstPool::new();
        pool.spawn(|_| panic!("whoa!"));
        println!("still ok");
        pool.send(1).expect("Sending failed");
        println!("still ok");
    }
}
