/// ------------Lamellar Test: SharedMemoryRegion collectives -------------------
/// Runs every SharedMemoryRegion collective (reduce, gather, broadcast, scatter, reduce-scatter; the plain, `_at_pe`
/// and `_into_buffer` forms) and prints each PE's result, one line per operation.
///
/// The output does not depend on the backend, so a backend that runs the collectives with the put/get algorithms
/// (rofi_c) can be checked against one that has them natively (shmem-opt) by sorting both outputs and comparing them.
/// Reductions are also checked here against values computed from the input.
///
/// usage: shared_mem_region_collectives_test <i32|u64|f64> <len>   (len must be a multiple of the number of PEs)
/// -----------------------------------------------------------------------------------------------
use lamellar::memregion::prelude::*;
use lamellar::{BroadcastInput, RootOrLamellarBuffer, RootSrcOrLamellarBuffer, ScatterInput};

macro_rules! collectives_test {
    ($t:ty, $int:expr, $len:expr) => {{
        let world = lamellar::LamellarWorldBuilder::new().build();
        let num_pes = world.num_pes();
        let my_pe = world.my_pe();
        let len: usize = $len;
        let n = len * num_pes;
        let root = 1 % num_pes;
        let mut out = String::new();
        let mut success = true;

        // element i of PE p is (i % 5 + 2) * (p + 1): small enough that the product over 4 PEs fits every type
        let value = |i: usize, pe: usize| ((i % 5 + 2) * (pe + 1)) as $t;
        let region: SharedMemoryRegion<$t> = world.alloc_shared_mem_region(n).block();
        unsafe {
            for (i, e) in region.as_mut_slice().iter_mut().enumerate() {
                *e = value(i, my_pe);
            }
        }
        world.barrier();

        macro_rules! record {
            ($name:expr, $v:expr) => {
                out.push_str(&format!("{} pe={} {:?}\n", $name, my_pe, $v));
            };
        }
        // the buffer a `_into_buffer` collective writes into, and a way to read it back
        let new_out = |size: usize| -> SharedMemoryRegion<$t> {
            let r: SharedMemoryRegion<$t> = world.alloc_shared_mem_region(size).block();
            unsafe { r.as_mut_slice().iter_mut().for_each(|e| *e = 0 as $t) };
            r
        };

        unsafe {
            // reductions over every PE, checked against the input
            let expect = |f: &dyn Fn(Vec<$t>) -> $t, start: usize| -> Vec<$t> {
                (0..len)
                    .map(|j| f((0..num_pes).map(|p| value(start + j, p)).collect()))
                    .collect()
            };
            let sum = region.sum_all(0, len).block();
            let prod = region.prod_all(0, len).block();
            let min = region.min_all(0, len).block();
            let max = region.max_all(0, len).block();
            for (name, got, want) in [
                ("sum_all", &sum, expect(&|v| v.iter().copied().sum(), 0)),
                ("prod_all", &prod, expect(&|v| v.iter().copied().product(), 0)),
                (
                    "min_all",
                    &min,
                    expect(&|v| v.iter().copied().fold(v[0], |a, b| if b < a { b } else { a }), 0),
                ),
                (
                    "max_all",
                    &max,
                    expect(&|v| v.iter().copied().fold(v[0], |a, b| if b > a { b } else { a }), 0),
                ),
            ] {
                if got.as_slice() != want.as_slice() {
                    eprintln!("{} wrong on pe {}: got {:?} expected {:?}", name, my_pe, got, want);
                    success = false;
                }
                record!(name, got);
            }
            // a window that does not start at the beginning of the region
            let sum_at = region.sum_all(len, len).block();
            if sum_at.as_slice() != expect(&|v| v.iter().copied().sum(), len).as_slice() {
                eprintln!("sum_all at index {} wrong on pe {}: {:?}", len, my_pe, sum_at);
                success = false;
            }
            record!("sum_all_at_index", sum_at);
            if $int {
                record!("bit_or_all", region.bit_or_all(0, len).block());
                record!("bit_xor_all", region.bit_xor_all(0, len).block());
                record!("bit_and_all", region.bit_and_all(0, len).block());
            }

            // reductions that end on one PE
            record!("sum_at_pe", region.sum_at_pe(0, len, root).block());
            record!("max_at_pe", region.max_at_pe(0, len, root).block());

            // gathers
            let gathered = region.gather_all(0, len).block();
            let want: Vec<$t> = (0..num_pes).flat_map(|p| (0..len).map(move |j| value(j, p))).collect();
            if gathered.as_slice() != want.as_slice() {
                eprintln!("gather_all wrong on pe {}: got {:?} expected {:?}", my_pe, gathered, want);
                success = false;
            }
            record!("gather_all", gathered);
            record!("gather_at_pe", region.gather_at_pe(0, len, root).block());
            record!("broadcast_all", region.broadcast_all(0, len).block());

            // broadcast and scatter from one PE
            let bcast = if my_pe == root {
                BroadcastInput::root(len)
            } else {
                BroadcastInput::not_root(root)
            };
            record!("broadcast_from_pe", region.broadcast_from_pe(bcast, len).block());
            let scat = if my_pe == root {
                ScatterInput::root(0)
            } else {
                ScatterInput::not_root(root)
            };
            record!("scatter_from_pe", region.scatter_from_pe(scat, len).block());

            // reduce-scatter
            record!("sum_scatter", region.sum_scatter(0, len).block());
            record!("max_scatter", region.max_scatter(0, len).block());

            // the _into_buffer forms
            let dst = new_out(len);
            region
                .sum_all_into_buffer(0, len, LamellarBuffer::from_shared_memory_region(dst.clone()))
                .block();
            record!("sum_all_into_buffer", dst.as_slice());

            let dst = new_out(n);
            region
                .gather_all_into_buffer(0, len, LamellarBuffer::from_shared_memory_region(dst.clone()))
                .block();
            record!("gather_all_into_buffer", dst.as_slice());

            let dst = new_out(len);
            let target = if my_pe == root {
                RootOrLamellarBuffer::Root(LamellarBuffer::from_shared_memory_region(dst.clone()))
            } else {
                RootOrLamellarBuffer::NotRoot(root)
            };
            region.sum_at_pe_into_buffer(0, len, target).block();
            record!("sum_at_pe_into_buffer", dst.as_slice());

            let dst = new_out(n);
            let target = if my_pe == root {
                RootOrLamellarBuffer::Root(LamellarBuffer::from_shared_memory_region(dst.clone()))
            } else {
                RootOrLamellarBuffer::NotRoot(root)
            };
            region.gather_at_pe_into_buffer(0, len, target).block();
            record!("gather_at_pe_into_buffer", dst.as_slice());

            let dst = new_out(len);
            let target = if my_pe == root {
                RootSrcOrLamellarBuffer::Root(len)
            } else {
                RootSrcOrLamellarBuffer::NotRoot(
                    LamellarBuffer::from_shared_memory_region(dst.clone()),
                    root,
                )
            };
            region.broadcast_from_pe_into_buffer(target, len).block();
            record!("broadcast_from_pe_into_buffer", dst.as_slice());

            let dst = new_out(len);
            let scat = if my_pe == root {
                ScatterInput::root(0)
            } else {
                ScatterInput::not_root(root)
            };
            region
                .scatter_from_pe_into_buffer(
                    LamellarBuffer::from_shared_memory_region(dst.clone()),
                    scat,
                    len,
                )
                .block();
            record!("scatter_from_pe_into_buffer", dst.as_slice());

            // each PE receives len / num_pes elements of the reduced window, so that is the size of the buffer
            let dst = new_out(len / num_pes);
            region
                .sum_scatter_into_buffer(0, len, LamellarBuffer::from_shared_memory_region(dst.clone()))
                .block();
            record!("sum_scatter_into_buffer", dst.as_slice());

            // a buffer longer than the share: the first len / num_pes elements are written, the rest is left as it was
            let dst = new_out(len);
            region
                .sum_scatter_into_buffer(0, len, LamellarBuffer::from_shared_memory_region(dst.clone()))
                .block();
            let share = len / num_pes;
            if dst.as_slice()[share..].iter().any(|e| *e != 0 as $t) {
                eprintln!("sum_scatter_into_buffer wrote beyond its share on pe {}: {:?}", my_pe, dst.as_slice());
                success = false;
            }
            record!("sum_scatter_into_buffer_longer", dst.as_slice());

            // the in-place forms: the buffer holds this PE's values going in and the reduction coming out, and the
            // region itself must be left as it was
            let fill = |dst: &SharedMemoryRegion<$t>| {
                dst.as_mut_slice().iter_mut().enumerate().for_each(|(i, e)| *e = value(i, my_pe));
            };
            let dst = new_out(len);
            fill(&dst);
            region
                .sum_all_in_place(LamellarBuffer::from_shared_memory_region(dst.clone()))
                .block();
            if dst.as_slice() != expect(&|v| v.iter().copied().sum(), 0).as_slice() {
                eprintln!("sum_all_in_place wrong on pe {}: {:?}", my_pe, dst.as_slice());
                success = false;
            }
            record!("sum_all_in_place", dst.as_slice());
            let dst = new_out(len);
            fill(&dst);
            region
                .max_all_in_place(LamellarBuffer::from_shared_memory_region(dst.clone()))
                .block();
            record!("max_all_in_place", dst.as_slice());
            if $int {
                let dst = new_out(len);
                fill(&dst);
                region
                    .bit_or_all_in_place(LamellarBuffer::from_shared_memory_region(dst.clone()))
                    .block();
                record!("bit_or_all_in_place", dst.as_slice());
            }
            record!("region_after_in_place", &region.as_slice()[..len]);
        }

        world.barrier();
        // one print per PE so the lines of different PEs do not interleave
        print!("{}", out);
        if !success {
            eprintln!("failed");
            std::process::exit(1);
        }
    }};
}

#[lamellar::main]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let elem = args[1].clone();
    let len = args[2].parse::<usize>().unwrap();

    match elem.as_str() {
        "i32" => collectives_test!(i32, true, len),
        "u64" => collectives_test!(u64, true, len),
        "f64" => collectives_test!(f64, false, len),
        _ => eprintln!("unsupported element type"),
    }
}
