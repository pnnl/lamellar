mod options;
use clap::Parser;

use lamellar::array::prelude::*;
use lamellar::memregion::prelude::*;
use rand::prelude::*;
use std::future::Future;
use std::time::Instant;

fn index_gather(
    array: &AtomicArray<usize>,
    rand_index: OneSidedMemoryRegion<usize>,
) -> impl Future<Output = Vec<usize>> {
    let rand_slice = unsafe { rand_index.as_slice().expect("PE on world team") }; // Safe as we are the only consumer of this mem region
    array.batch_load(rand_slice)
}

// srun -N <num nodes> target/release/histo_lamellar_array <num updates>
fn main() {
    let world = lamellar::LamellarWorldBuilder::new().build();
    let my_pe = world.my_pe();
    let num_pes = world.num_pes();
    let cli = options::IndexGatherCli::parse();

    let global_count = cli.global_size;
    let g_num_updates = cli.global_updates;
    let l_num_updates = g_num_updates / num_pes;
    let iterations = cli.iterations;

    if my_pe == 0 {
        cli.describe(num_pes);
    }

    let unsafe_array = UnsafeArray::<usize>::new(
        world.team(),
        global_count,
        lamellar::array::Distribution::Cyclic,
    );
    let rand_index = world.alloc_one_sided_mem_region(l_num_updates);
    let mut rng: StdRng = SeedableRng::seed_from_u64(my_pe as u64);

    // initialize arrays
    let array_init = unsafe {
        unsafe_array
            .dist_iter_mut()
            .enumerate()
            .for_each(|(i, x)| *x = i)
    };
    unsafe {
        for elem in rand_index.as_mut_slice().iter_mut() {
            *elem = rng.gen_range(0, global_count);
        }
    }
    world.block_on(array_init);
    let array = unsafe_array.into_atomic();
    for _i in 0..iterations {
        world.barrier();

        if my_pe == 0 {
            println!("starting index gather");
        }

        let now = Instant::now();
        let res = index_gather(&array, rand_index.clone());

        if my_pe == 0 {
            println!("{:?} issue time {:?} ", my_pe, now.elapsed());
        }
        array.block_on(res);
        if my_pe == 0 {
            println!(
                "local run time {:?} local mups: {:?}",
                now.elapsed(),
                (l_num_updates as f32 / 1_000_000.0) / now.elapsed().as_secs_f32()
            );
        }
        array.barrier();
        let global_time = now.elapsed().as_secs_f64();
        if my_pe == 0 {
            println!(
                "global time {:?} MB {:?} MB/s: {:?}",
                global_time,
                (world.MB_sent()),
                (world.MB_sent()) / global_time,
            );
            println!(
                "MUPS: {:?}",
                ((l_num_updates * num_pes) as f64 / 1_000_000.0) / global_time,
            );
            println!("Secs: {:?}", global_time,);
            println!(
                "GB/s Injection rate: {:?}",
                (8.0 * (l_num_updates * 2) as f64 * 1.0E-9) / global_time,
            );
        }
    }
}
