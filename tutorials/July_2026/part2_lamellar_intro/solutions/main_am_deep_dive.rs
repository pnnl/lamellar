use lamellar::active_messaging::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

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

    if my_pe == 0 {
        let request = world.spawn_am_pe(0, SquareAm { val: 7 });
        let result = request.block();
        println!("PE 0: square(7) = {result}");
    }
    world.barrier();

    let data = Arc::new((0..1000).collect::<Vec<usize>>());
    let total = Arc::new(AtomicUsize::new(0));
    let num_threads = std::cmp::max(world.num_threads_per_pe(), 1);
    let chunk = data.len() / num_threads;

    for t in 0..num_threads {
        let start = t * chunk;
        let end = if t == num_threads - 1 { data.len() } else { start + chunk };
        let _ = world
            .spawn_am_local(LocalSumAm {
                data: data.clone(),
                start,
                end,
                total: total.clone(),
            })
            .spawn(); // launch now, without blocking this loop iteration
    }
    world.wait_all(); // wait for every local AM to finish before reading `total`

    println!("PE {my_pe}: partial total = {:?}", total.load(Ordering::Relaxed));
}
