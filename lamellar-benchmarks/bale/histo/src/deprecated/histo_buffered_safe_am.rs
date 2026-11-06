mod options;
use clap::Parser;

use lamellar::active_messaging::prelude::*;
use lamellar::darc::prelude::*;
use lamellar::memregion::prelude::*;

use rand::prelude::*;
use std::time::Instant;

use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};

//===== HISTO BEGIN ======

#[lamellar::AmData(Clone, Debug)]
struct HistoBufferedAM {
    buff: std::vec::Vec<usize>,
    counts: Darc<Vec<AtomicUsize>>,
}

#[lamellar::am]
impl LamellarAM for HistoBufferedAM {
    async fn exec(self) {
        for o in &self.buff {
            self.counts[*o].fetch_add(1, Ordering::Relaxed);
        }
    }
}

#[lamellar::AmLocalData(Clone, Debug)]
struct LaunchAm {
    rand_index: OneSidedMemoryRegion<usize>,
    counts: Darc<Vec<AtomicUsize>>,
    buffer_size: usize,
}

#[lamellar::local_am]
impl LamellarAM for LaunchAm {
    async fn exec(self) {
        let num_pes = lamellar::num_pes;
        let mut buffs: std::vec::Vec<std::vec::Vec<usize>> =
            vec![Vec::with_capacity(self.buffer_size); num_pes];
        let task_group = LamellarTaskGroup::new(lamellar::team.clone());
        for idx in unsafe { self.rand_index.as_slice() } {
            let rank = idx % num_pes;
            let offset = idx / num_pes;
            buffs[rank].push(offset);
            if buffs[rank].len() >= self.buffer_size {
                let buff = buffs[rank].clone();
                task_group.exec_am_pe(
                    rank,
                    HistoBufferedAM {
                        buff: buff,
                        counts: self.counts.clone(),
                    },
                );
                buffs[rank].clear();
            }
        }
        //send any remaining buffered updates
        let _timer = Instant::now();
        for (rank, buff) in buffs.into_iter().enumerate() {
            if buff.len() > 0 {
                task_group.exec_am_pe(
                    rank,
                    HistoBufferedAM {
                        buff: buff,
                        counts: self.counts.clone(),
                    },
                );
            }
        }
    }
}

fn histo(
    l_num_updates: usize,
    launch_threads: usize,
    world: &LamellarWorld,
    rand_index: &OneSidedMemoryRegion<usize>,
    counts: &Darc<Vec<AtomicUsize>>,
    buffer_size: usize,
) -> Vec<impl Future<Output = ()>> {
    let slice_size = l_num_updates as f32 / launch_threads as f32;
    let mut launch_tasks = vec![];
    for tid in 0..launch_threads {
        let start = (tid as f32 * slice_size).round().round() as usize;
        let end = ((tid + 1) as f32 * slice_size).round() as usize;
        launch_tasks.push(world.exec_am_local(LaunchAm {
            rand_index: rand_index.sub_region(start..end),
            counts: counts.clone(),
            buffer_size: buffer_size,
        }));
    }
    launch_tasks
}

//===== HISTO END ======

fn main() {
    let world = lamellar::LamellarWorldBuilder::new().build();
    let my_pe = world.my_pe();
    let num_pes = world.num_pes();
    let cli = options::HistoCli::parse();

    let global_count = cli.global_size;
    let local_count = (global_count as f32 / num_pes as f32).ceil() as usize; // round up just to be safe that we have enough space on each pe
    let g_num_updates = cli.global_updates;
    let l_num_updates = g_num_updates / num_pes;
    let iterations = cli.iterations;
    let launch_threads = cli.launch_threads;
    let buffer_size = cli.buffer_size;

    if my_pe == 0 {
        cli.describe(num_pes);
    }

    let mut counts_data = Vec::with_capacity(local_count);
    for _ in 0..local_count {
        counts_data.push(AtomicUsize::new(0));
    }
    let counts = Darc::new(&world, counts_data).expect("unable to create darc");

    let rand_index = world.alloc_one_sided_mem_region(l_num_updates);
    let mut rng: StdRng = SeedableRng::seed_from_u64(my_pe as u64);

    unsafe {
        for elem in rand_index.as_mut_slice().iter_mut() {
            *elem = rng.gen_range(0, global_count);
        }
    }

    for _i in 0..iterations {
        //get number of updates to perform from first command line argument otherwise set to 1000 updates
        world.barrier();
        let now = Instant::now();
        let launch_tasks = histo(
            l_num_updates,
            launch_threads,
            &world,
            &rand_index,
            &counts,
            buffer_size,
        );

        if my_pe == 0 {
            println!("{:?} issue time {:?} ", my_pe, now.elapsed(),);
        }
        world.block_on(async move { futures::future::join_all(launch_tasks).await });
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
            println!("Secs: {:?}", global_time,);
            println!(
                "GB/s Injection rate: {:?}",
                (8.0 * (l_num_updates * 2) as f64 * 1.0E-9) / global_time,
            );
        }

        if my_pe == 0 {
            println!(
                "{:?} global time {:?} MB {:?} MB/s: {:?} global mups: {:?} (({l_num_updates}*{num_pes})/1_000_000) ",
                my_pe,
                global_time,
                world.MB_sent(),
                world.MB_sent() / global_time,
                ((l_num_updates * num_pes) as f64 / 1_000_000.0) / global_time,

            );
        }
        if my_pe == 0 {
            println!(
                "pe {:?} sum {:?}",
                my_pe,
                counts
                    .iter()
                    .map(|e| e.load(Ordering::Relaxed))
                    .sum::<usize>()
            );
        }
        for elem in counts.iter() {
            elem.store(0, Ordering::SeqCst);
        }
    }
}
