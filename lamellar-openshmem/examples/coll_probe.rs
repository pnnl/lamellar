//! Times the Lamellar runtime's own collectives (barrier, sum_all, broadcast_from_pe, gather_all on a
//! GlobalLockArray) so they can be compared with the shim's hand-written OpenSHMEM collectives.
//! Run under oshrun with LAMELLAR_EXECUTOR=single_thread. Output: `CSV,probe,<bench>,<bytes>,<np>,<p50_us>,<max_us>`.
//! UCC_LOG_LEVEL / UCC_COLL_TRACE show whether (and how) UCC runs each operation.
use lamellar::array::prelude::*;
use lamellar::memregion::prelude::*;
use std::time::Instant;

const MAXN: usize = 8192; // u64 elements per PE (64 KiB)

fn stats(mut v: Vec<f64>) -> (f64, f64) {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    (v[v.len() / 2], v[v.len() - 1])
}

fn main() {
    let world = LamellarWorldBuilder::new().build();
    let (me, np) = (world.my_pe(), world.num_pes());
    let iters: usize = std::env::var("PROBE_ITERS").ok().and_then(|s| s.parse().ok()).unwrap_or(200);
    let array: GlobalLockArray<u64> =
        GlobalLockArray::new(&world, np * MAXN, Distribution::Block).block();
    let guard = array.collective_write_local_data().block();
    // the same collectives called directly on a SharedMemoryRegion (what the shim's own heap region is)
    let mr: SharedMemoryRegion<u64> = world.alloc_shared_mem_region(MAXN).block();
    // Root(i): this PE is the root and the data is at local index i; NotRoot(pe): the root is `pe`
    let bin = || if me == 0 { BroadcastInput::Root(0) } else { BroadcastInput::NotRoot(0) };
    // per-iteration time of the slowest PE (a root-only timing would stop the clock when the root's sends
    // are issued, not when the last PE has the data): element-wise max of every PE's times
    let scratch: SharedMemoryRegion<u64> = world.alloc_shared_mem_region(iters).block();
    let slowest = |t: &[f64]| -> Vec<f64> {
        unsafe {
            let sl = scratch.as_mut_slice();
            for (i, v) in t.iter().enumerate() {
                sl[i] = (*v * 1000.0) as u64; // ns
            }
            scratch.max_all(0, t.len()).block().into_iter().map(|n| n as f64 / 1000.0).collect()
        }
    };
    let report = |name: &str, bytes: usize, t: Vec<f64>| {
        let t = slowest(&t);
        if me == 0 {
            let (p50, mx) = stats(t);
            println!("CSV,probe,{name},{bytes},{np},{p50:.2},{mx:.2}");
        }
    };
    // time one operation per iteration on PE 0, with a barrier before each so PEs start together
    macro_rules! time {
        ($name:expr, $bytes:expr, $op:expr) => {{
            for _ in 0..10 {
                world.barrier();
                $op;
            }
            let mut t = Vec::with_capacity(iters);
            for _ in 0..iters {
                world.barrier();
                let s = Instant::now();
                $op;
                t.push(s.elapsed().as_secs_f64() * 1e6);
            }
            report($name, $bytes, t);
        }};
    }
    time!("barrier", 0, world.barrier());
    for &n in &[1usize, 128, MAXN] {
        let b = n * 8;
        time!("sum_all", b, { std::hint::black_box(guard.sum_all(0, n).block()); });
        time!("broadcast", b, { std::hint::black_box(guard.broadcast_from_pe(bin(), n).block()); });
        time!("gather_all", b, { std::hint::black_box(guard.gather_all(0, n).block()); });
        time!("region_sum_all", b, { std::hint::black_box(unsafe { mr.sum_all(0, n) }.block()); });
        time!("region_broadcast", b, { std::hint::black_box(unsafe { mr.broadcast_from_pe(bin(), n) }.block()); });
        time!("region_gather_all", b, { std::hint::black_box(unsafe { mr.gather_all(0, n) }.block()); });
    }
}
