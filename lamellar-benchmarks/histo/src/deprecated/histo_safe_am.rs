mod options;
use clap::Parser;

use lamellar::active_messaging::prelude::*;
use lamellar::memregion::prelude::*;

use rand::prelude::*;
use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

//===== HISTO BEGIN ======

#[lamellar::AmData(Clone, Debug)]
struct HistoAM {
    offset: usize,
    counts: SharedMemoryRegion<usize>,
}

#[lamellar::am]
impl LamellarAM for HistoAM {
    async fn exec(self) {
        // this casts the underlying entry to an atomicusize to perform atomic updates
        let elem = unsafe {
            ((&mut self.counts.as_mut_slice()[self.offset] as *mut usize)
                as *mut AtomicUsize)
                .as_ref()
                .unwrap()
        };
        elem.fetch_add(1, Ordering::Relaxed);
    }
}

#[lamellar::AmLocalData(Clone, Debug)]
struct LaunchAm {
    rand_index: OneSidedMemoryRegion<usize>,
    counts: SharedMemoryRegion<usize>,
}

#[lamellar::local_am]
impl LamellarAM for LaunchAm {
    async fn exec(self) {
        for idx in unsafe { self.rand_index.as_slice() } {
            let rank = idx % lamellar::num_pes;
            let offset = idx / lamellar::num_pes;
            lamellar::world.exec_am_pe(
                rank,
                HistoAM {
                    offset: offset,
                    counts: self.counts.clone(),
                },
            );
        }
    }
}

fn histo(
    l_num_updates: usize,
    launch_threads: usize,
    world: &LamellarWorld,
    rand_index: &OneSidedMemoryRegion<usize>,
    counts: &SharedMemoryRegion<usize>,
) -> Vec<impl Future<Output = ()>> {
    let slice_size = l_num_updates as f32 / launch_threads as f32;
    let mut launch_tasks = vec![];
    for tid in 0..launch_threads {
        let start = (tid as f32 * slice_size).round().round() as usize;
        let end = ((tid + 1) as f32 * slice_size).round() as usize;
        launch_tasks.push(world.exec_am_local(LaunchAm {
            rand_index: rand_index.sub_region(start..end),
            counts: counts.clone(),
        }));
    }
    launch_tasks
}

//===== HISTO END ======

// srun -N <num nodes> target/release/histo <num updates>
fn main() {
    let world = lamellar::LamellarWorldBuilder::new().build();
    let my_pe = world.my_pe();
    let num_pes = world.num_pes();
    let cli = options::HistoCli::parse();

    let global_count = cli.global_size;
    let local_count = (global_count as f32 / num_pes as f32).ceil() as usize; // round up just to be safe that we have enough space on each pe
    let g_num_updates = cli.global_updates;
    let l_num_updates = g_num_updates / num_pes;
    let launch_threads = cli.launch_threads;

    if my_pe == 0 {
        cli.describe(num_pes);
    }

    let counts = world.alloc_shared_mem_region(local_count).block();
    let rand_index = world.alloc_one_sided_mem_region(l_num_updates);
    let mut rng: StdRng = SeedableRng::seed_from_u64(my_pe as u64);
    //initialize arrays
    unsafe {
        for elem in counts.as_mut_slice().iter_mut() {
            *elem = 0;
        }
        for elem in rand_index.as_mut_slice().iter_mut() {
            *elem = rng.gen_range(0, global_count);
        }
    }

    //create multiple launch tasks, that iterated through portions of rand_index in parallel

    world.barrier();
    let now = Instant::now();
    let launch_tasks = histo(l_num_updates, launch_threads, &world, &rand_index, &counts);

    if my_pe == 0 {
        println!("{:?} issue time {:?} ", my_pe, now.elapsed());
    }
    world.block_on(async move {
        for task in launch_tasks {
            task.await;
        }
    });
    if my_pe == 0 {
        println!("{:?} launch task time {:?} ", my_pe, now.elapsed(),);
    }

    world.wait_all();
    if my_pe == 0 {
        println!(
            "local run time {:?} local mups: {:?}",
            now.elapsed(),
            (l_num_updates as f32 / 1_000_000.0) / now.elapsed().as_secs_f32()
        );
    }
    world.barrier();
    let global_time = now.elapsed().as_secs_f64();
    if my_pe == 0 {
        println!(
            "MUPS: {:?}",
            ((l_num_updates * num_pes) as f64 / 1_000_000.0) / global_time
        );
    }
    if my_pe == 0 {
        println!(
            "{:?} global time {:?} MB {:?} MB/s: {:?} global mups: {:?} ",
            my_pe,
            global_time,
            world.MB_sent(),
            world.MB_sent() / global_time,
            ((l_num_updates * num_pes) as f64 / 1_000_000.0) / global_time
        );
    }

    println!("pe {:?} sum {:?}", my_pe, unsafe {
        counts.as_slice().iter().sum::<usize>()
    });
}
