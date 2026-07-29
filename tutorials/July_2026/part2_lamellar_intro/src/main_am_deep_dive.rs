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
        // spawn_am_local is EAGER - it's already submitted to the scheduler here,
        // dropping the handle doesn't stop it from running.
        let _ = world.spawn_am_local(LocalSumAm {
            data: data.clone(),
            start,
            end,
            total: total.clone(),
        });
    }

    // BUG: nothing waits for the fanned-out LocalSumAm's to finish before reading
    // `total` - a race, so the print below may show a partial (or even 0) sum
    // depending on how fast the AMs land relative to this line.
    println!("PE {my_pe}: partial total = {:?}", total.load(Ordering::Relaxed));
}
