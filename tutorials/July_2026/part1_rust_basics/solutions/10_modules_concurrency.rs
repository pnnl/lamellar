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

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

fn main() {
    let counter = counters::Counter::new();
    println!("start value: {}", counter.value);

    let shared = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    for _ in 0..4 {
        let shared = shared.clone(); // clone the Arc (bumps refcount), not the underlying data
        handles.push(thread::spawn(move || {
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
