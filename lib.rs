/*! A thread pool optimised for bursts of activity.

A single thread produces work which is then performed by a pool of worker threads. We expect the
work to be produces infrequently, but in bursts. This means that, in normal operation, all threads
in the pool sleep for some period of time, until some event requires some or all of them to be
suddenly be woken at once. The threads then perform some work before all going back to sleep again.

Normally thread pools schedule work based on thread readiness, and are implemented using a
work-stealing queue. However, this doesn't work well for our use case, since many threads become
runnable at the same time and subsequently create a lot of contention when reading from the queue.
Instead, we use a round-robin scheduling strategy to send work to specific threads, which
eliminates contention at the cost of performing badly in the case where the workloads are uneven
and utilisation is high.

```
use burst_pool::BurstPool;

let mut pool = BurstPool::new();
for tid in 0..3 {
    pool.spawn(move|iter| {
        println!("Thread {} received {}", tid, iter);
    });
}

for i in 0..5 {
    pool.send(i);
}
```

Gives

```none
Thread 0 received 0
Thread 0 received 3
Thread 1 received 1
Thread 1 received 4
Thread 2 received 2
```

## Performance

A pool of 10 threads, to which 10 messages are sent. The threads all wake, read the clock, and then
go back to sleep. On my machine (which has `max_cstate = 0` - this makes a huge difference to
thread wake-up times), I see the following results:

```none
 time (us): frequency
----------:-----------
       0-1:     0
       1-2:     3
       2-2:    57 ++
       2-3:   304 ++++++++++++++
       3-5:   808 +++++++++++++++++++++++++++++++++++++
       5-8:  1289 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
      8-11:  1491 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
     11-17:   831 +++++++++++++++++++++++++++++++++++++++
     17-26:   184 ++++++++
     26-38:    23 +
     38-58:     7
     58-86:     2
    86-130:     1
      130+:     0

(12.7 us mean)
(5000 samples total)
```
*/

#[macro_use] extern crate log;

use std::thread::{self,JoinHandle};
use std::sync::mpsc;

/// A thread pool optimised for bursts of activity.
///
/// A `BurstPool` maintains one unbounded channel (from sync::mpsc) for each thread. Because these
/// channels have only one producer, the more efficient spsc implementation will be used.
///
/// Threads spawned in the pool will block (*not* busy-wait) until there is work to be done. When
/// `BurstPool::send()` is called, a thread is chosen, the payload is pushed onto its queue, and it
/// is unparked. Threads are selected round-robin, so uneven processing time is not handled well.
///
/// When the `BurstPool` is dropped, all threads will be signalled to exit. Dropping will block
/// until the threads are all dead.
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
