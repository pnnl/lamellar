// `mod` groups related code into a namespace. Lamellar itself is organized this way
// (e.g. lamellar::array, lamellar::active_messaging).
mod counters {
    pub struct Counter {
        pub value: usize,
    }

    impl Counter {
        pub fn new() -> Self {
            Counter { value: 0 }
        }
    }
}

// Arc<T> = Atomically Reference Counted pointer: shared ownership across threads.
// This is the local (single-process) analog of Lamellar's `Darc<T>` in part 2 -
// same idea (shared ownership + safe concurrent access), but Darc works across PEs
// (separate processes/nodes) instead of just across threads in one process.
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

fn main() {
    let counter = counters::Counter::new();
    println!("start value: {}", counter.value);

    let shared = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    for _ in 0..4 {
        handles.push(thread::spawn(move || {
            // error: `shared` moved into the first spawned closure,
            // then used again on the next loop iteration
            for _ in 0..1000 {
                shared.fetch_add(1, Ordering::Relaxed);
            }
        }));
    }

    for h in handles {
        let _ = h.join();
    }

    println!("final count: {}", shared.load(Ordering::Relaxed));
}
