// Part 2, Section 5: Array type survey.
// Spectrum: UnsafeArray (no guarantees) -> ReadOnlyArray / AtomicArray / LocalLockArray /
// GlobalLockArray (increasing safety, decreasing raw throughput).
// Intentional bug: see BUG comment below.
use lamellar::array::prelude::*;

#[lamellar::main]
fn main() {
    let world = LamellarWorldBuilder::new().build();
    let my_pe = world.my_pe();
    let num_pes = world.num_pes();
    let len = num_pes * 4;

    // UnsafeArray: the foundation every other array type is built from/on top of.
    // No access control - PEs may read/write anywhere with no synchronization.
    let array = UnsafeArray::<usize>::new(world.team(), len, Distribution::Block).block();
    unsafe {
        array
            .dist_iter_mut()
            .enumerate()
            .for_each(move |(i, elem)| *elem = i)
            .block();
    }
    world.barrier();

    // Convert to AtomicArray: per-element atomic read/write, safe for concurrent access
    // from multiple PEs/threads without external locking.
    let atomic_array = array.into_atomic().block();
    atomic_array
        .dist_iter()
        .for_each(move |elem| {
            elem.fetch_add(my_pe);
        })
        .block();
    world.barrier();

    // BUG: into_read_only() takes `self` by value, consuming atomic_array. Dropping the
    // returned handle instead of binding it doesn't leave the old handle usable - it's a
    // hard compile error (E0382, "borrow of moved value") on the `atomic_array` use below.
    // Fix by binding (and blocking) the result: `let read_only_array = atomic_array.into_read_only().block();`
    atomic_array.into_read_only();

    if my_pe == 0 {
        atomic_array
            .onesided_iter()
            .into_iter()
            .for_each(|elem| print!("{elem} "));
        println!();
    }

    // Not shown here, same conversion pattern applies to:
    //   array.into_local_lock().block()  -> LocalLockArray (per-PE RwLock)
    //   array.into_global_lock().block() -> GlobalLockArray (single global RwLock)
    // Use these when you need multi-element atomic-like transactions that a plain
    // AtomicArray (per-element only) can't express.
}
