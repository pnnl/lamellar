// Part 2, Section 4: AM deep dive - return values + local-only AMs.
// Intentional bug: see BUG comment below.
use lamellar::active_messaging::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// AM that returns a value back to the caller.
#[AmData(Debug, Clone)]
struct SquareAm {
    val: usize,
}

#[am]
impl LamellarAM for SquareAm {
    async fn exec(self) -> usize {
        self.val * self.val
    }
}

// Local-only AM: no serialization needed since it never leaves the issuing PE.
// Useful for spreading work across a PE's own worker threads.
#[AmLocalData(Clone)]
struct LocalSumAm {
    data: Arc<Vec<usize>>,
    start: usize,
    end: usize,
    total: Arc<AtomicUsize>,
}

#[local_am]
impl LamellarAM for LocalSumAm {
    async fn exec(self) {
        let partial: usize = self.data[self.start..self.end].iter().sum();
        self.total.fetch_add(partial, Ordering::Relaxed);
    }
}

#[lamellar::main]
fn main() {
    let world = LamellarWorldBuilder::new().build();
    let my_pe = world.my_pe();
    world.barrier();

    // --- remote AM returning a value ---
    if my_pe == 0 {
        let request = world.spawn_am_pe(0, SquareAm { val: 7 });
        let result = request.block();
        println!("PE 0: square(7) = {result}");
    }
    world.barrier();

    // --- local AM fan-out across worker threads ---
    let data = Arc::new((0..1000).collect::<Vec<usize>>());
    let total = Arc::new(AtomicUsize::new(0));
    let num_threads = std::cmp::max(world.num_threads_per_pe(), 1);
    let chunk = data.len() / num_threads;

    for t in 0..num_threads {
        let start = t * chunk;
        let end = if t == num_threads - 1 { data.len() } else { start + chunk };
        // BUG: spawn_am_local returns a lazy future - it does nothing unless you call
        // .spawn() or .block() on it. Dropping the result here means the AM never runs
        // at all, so `total` stays 0.
        let _ = world.spawn_am_local(LocalSumAm {
            data: data.clone(),
            start,
            end,
            total: total.clone(),
        });
    }

    println!("PE {my_pe}: partial total = {:?}", total.load(Ordering::Relaxed));
}
