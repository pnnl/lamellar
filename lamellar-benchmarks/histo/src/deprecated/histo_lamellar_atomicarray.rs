mod options;
use clap::Parser;

use lamellar::active_messaging::prelude::*;
use lamellar::array::prelude::*;

use parking_lot::Mutex;
use rand::prelude::*;
use std::future::Future;
use std::sync::Arc;
use std::time::Instant;

//===== HISTO BEGIN ======

fn histo(
    counts: &AtomicArray<usize>,
    rand_index: &ReadOnlyArray<usize>,
    now: &Instant,
) -> impl Future<Output = ()> {
    let res = counts.batch_add(rand_index.local_data(), 1);
    if counts.my_pe() == 0 {
        println!("{:?} issue time {:?} ", counts.my_pe(), now.elapsed());
    }
    res
}

//===== HISTO END ======

// srun -N <num nodes> target/release/histo_lamellar_array <num updates>
fn main() {
    let world = lamellar::LamellarWorldBuilder::new().build();
    let my_pe = world.my_pe();
    let num_pes = world.num_pes();
    let cli = options::HistoCli::parse();

    let global_count = cli.global_size;
    let g_num_updates = cli.global_updates;
    let l_num_updates = g_num_updates / num_pes;
    let iterations = cli.iterations;

    if my_pe == 0 {
        cli.describe(num_pes);
    }

    let unsafe_counts = UnsafeArray::<usize>::new(
        world.team(),
        global_count,
        lamellar::array::Distribution::Block,
    );
    let rand_index = UnsafeArray::<usize>::new(
        world.team(),
        l_num_updates * num_pes,
        lamellar::array::Distribution::Block,
    );
    let rng: Arc<Mutex<StdRng>> = Arc::new(Mutex::new(SeedableRng::seed_from_u64(my_pe as u64)));

    // initialize arrays
    let counts_init = unsafe { unsafe_counts.dist_iter_mut().for_each(|x| *x = 0) };
    // rand_index.dist_iter_mut().for_each(move |x| *x = rng.lock().gen_range(0,global_count)).wait(); //this is slow because of the lock on the rng so we will do unsafe slice version instead...
    unsafe {
        let mut rng = rng.lock();
        for elem in rand_index.local_as_mut_slice().iter_mut() {
            *elem = rng.gen_range(0, global_count);
        }
    }
    world.block_on(counts_init);
    let counts = unsafe_counts.into_atomic();
    let rand_index = rand_index.into_read_only();

    for _i in 0..iterations {
        if my_pe == 0 {
            println!("iteration {} of {iterations}", _i);
        }
        world.block_on(counts.dist_iter_mut().for_each(|x| x.store(0)));
        world.barrier();

        let now = Instant::now();

        counts.block_on(histo(&counts, &rand_index, &now));
        if my_pe == 0 {
            println!(
                "local run time {:?} local mups: {:?}",
                now.elapsed(),
                (l_num_updates as f32 / 1_000_000.0) / now.elapsed().as_secs_f32()
            );
        }
        counts.barrier();
        let global_time = now.elapsed().as_secs_f64();
        if my_pe == 0 {
            println!(
                "global time {:?} MB {:?} MB/s: {:?} ",
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
            println!("(({l_num_updates}*{num_pes})/1_000_000) ");
            println!("pe {:?} sum {:?}", my_pe, world.block_on(counts.sum()));
        }
    }
}
