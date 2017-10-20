/*!
BurstChan is an spmc channel designed for cases where values must be processed now or never.

`send` will only succeed if there is a receiver ready to take the value *right now*. If no
receivers are ready, the value is returned-to-sender.
*/

extern crate nix;
extern crate byteorder;

use byteorder::*;
use nix::sys::eventfd::*;
use nix::unistd::*;
use std::os::unix::io::RawFd;
use std::ptr;
use std::sync::atomic::*;
use std::sync::Arc;

pub struct Sender<T> {
    eventfd: RawFd,
    workers: Vec<Arc<Worker<T>>>,
    next_worker: usize,
    eventfd_buf: [u8; 8],
}

pub struct Receiver<T> {
    inner: Arc<Worker<T>>,
    eventfd: RawFd,
    eventfd_buf: [u8; 8],
}

unsafe impl<T: Send> Send for Sender<T> {}
unsafe impl<T: Send> Send for Receiver<T> {}

struct Worker<T> {
    state: AtomicUsize,
    slot: AtomicPtr<T>,
}

// Receiver states
const RS_WAITING:  usize = 0;  // This receiver has no work to do, and is parked
const RS_PENDING:  usize = 1;  // This receiver has work to do, but hasn't unparked yet
const RS_WORKING:  usize = 2;  // This receiver is unparked and is doing some work
const RS_ORPHANED: usize = 3;  // The sender has gone away, never to return.

impl<T> Sender<T> {
    pub fn new() -> Sender<T> {
        Sender {
            eventfd: eventfd(0, EFD_SEMAPHORE).unwrap(),
            workers: vec![],
            next_worker: 0,
            eventfd_buf: [0;8],
        }
    }

    pub fn receiver(&mut self) -> Receiver<T> {
        let worker = Arc::new(Worker {
            state: AtomicUsize::new(RS_WORKING),
            slot: AtomicPtr::new(ptr::null_mut()),
        });
        self.workers.push(worker.clone());
        Receiver {
            inner: worker,
            eventfd: self.eventfd,
            eventfd_buf: [0; 8],
        }
    }

    /// Guaranteed to wake up all the threads. If some threads are already running, then some
    /// threads may be woken up spuriously in the future as a result.
    pub fn unblock_all(&mut self) {
        NativeEndian::write_u64(&mut self.eventfd_buf[..], self.workers.len() as u64);
        write(self.eventfd, &self.eventfd_buf).unwrap();
    }
}

impl<T: Send> Sender<T> {
    /// Attempt to send `x` to one of the recievers.
    ///
    /// A receiver which is currently blocked on a call to `recv` will unblock, recieving `x`. If
    /// no such recievers exist, this function will return `x` to the caller. This never blocks.
    pub fn send(&mut self, x: Box<T>) -> Option<Box<T>> {
        let x = self.send_nounblock(x);
        self.unblock_all();
        x
    }

    pub fn send_nounblock(&mut self, x: Box<T>) -> Option<Box<T>> {
        // 1. Find a receiver in WAITING state
        // 2. Write ptr to that receiver's slot
        // 3. Set that receiver to PENDING state

        let mut target_worker = None;
        for i in 0..self.workers.len() {
            let i2 = (i + self.next_worker) % self.workers.len();
            match self.workers[i2].state.compare_and_swap(RS_WAITING, RS_PENDING, Ordering::Relaxed) {
                RS_WAITING => {
                    /* it was ready */
                    target_worker = Some(i2);
                    break;
                }
                RS_PENDING | RS_WORKING => { /* it's busy */ }
                _ => { /* logic error! */ panic!("send: bad state"); }
            }
        }
        match target_worker {
            Some(i) => {
                let ptr = self.workers[i].slot.swap(Box::into_raw(x), Ordering::Relaxed);
                assert!(ptr.is_null(), "send: non-null ptr");
                self.next_worker = (i + 1) % self.workers.len();
                None
            }
            None => Some(x)
        }
    }
}

impl<T> Receiver<T> {
    /// Blocks until send() is called on the sender.
    pub fn recv(&mut self) -> Box<T> {
        // 1. Set state to WAITING
        // 2. Block on eventfd
        // 3. Check state to make sure it's PENDING
        // 4. If so, change state to WORKING
        // 5. Take ptr from slot and return

        // At this point, the state should be RS_WORKING. Anything else is a logic error.
        match self.inner.state.swap(RS_WAITING, Ordering::Relaxed) {
            RS_WORKING => { /* expected */ }
            _ => { /* logic error! */ panic!("recv: bad state"); }
        }
        loop {
            read(self.eventfd, &mut self.eventfd_buf).unwrap();
            match self.inner.state.compare_and_swap(RS_PENDING, RS_WORKING, Ordering::Relaxed) {
                RS_PENDING => { /* this was a genuine wakeup. let's do some work! */ break; }
                RS_WAITING => { /* this was a spurious wakeup. let's go back to sleep. */ }
                _ => { /* logic error! */ panic!("recv: bad state"); }
            }
        }
        let ptr = self.inner.slot.swap(ptr::null_mut(), Ordering::Relaxed);
        assert!(!ptr.is_null(), "recv: null ptr");
        unsafe { Box::from_raw(ptr) }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        // Inform the receivers that the sender is going away.
        for w in self.workers.drain(..) {
            w.state.store(RS_ORPHANED, Ordering::Relaxed);
        }
        self.unblock_all();
    }
}

#[cfg(test)]
mod tests {
}
