/*! A thread pool optimised for bursts of activity.

The target use-case is one where all threads sleep for some period of time, but then a large number
of them must suddenly be woken at once. Normally thread pools are implemented with a work-stealing
queue, but in this use-case we are guaranteed to create lots of contention on the queue.

```
use burst_pool::BurstPool;
use std::time::Instant;

let mut pool = BurstPool::new();
for _ in 0..10 {
    pool.spawn(|ts: Instant| {
        println!("{:?}", ts.elapsed());
    });
}

let now = Instant::now();
for _ in 0..10 { pool.send(now); }
```

(Note that the above example will actually perform quite badly, because `println!` involves taking
a global lock.)
*/

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
    pub threads: Vec<(JoinHandle<()>, mpsc::Sender<T>)>,
    pub next_thread: usize,
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
    pub fn send(&mut self, x: T) {
        {
            let (ref mut handle, ref mut tx) = self.threads[self.next_thread];
            tx.send(x).unwrap();
            handle.thread().unpark();
        }
        self.next_thread = (self.next_thread + 1) % self.threads.len();
    }
}

impl<T> Drop for BurstPool<T> {
    /// Signal all the threads in the pool to shut down, and wait for them to do so.
    fn drop(&mut self) {
        for (handle, sender) in self.threads.drain(..) {
            ::std::mem::drop(sender);
            handle.thread().unpark();
            handle.join().unwrap();
        }
    }
}
