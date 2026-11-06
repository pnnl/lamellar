mod options;
use clap::Parser;

use lamellar::active_messaging::prelude::*;
use lamellar::darc::prelude::*;

use rand::prelude::*;
use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

//===== HISTO BEGIN ======

#[lamellar::AmData(Clone, Debug)]
struct HistoAM {
    offset: usize,
    counts: Darc<Vec<AtomicUsize>>,
}

#[lamellar::am]
impl LamellarAM for HistoAM {
    async fn exec(self) {
        self.counts[self.offset].fetch_add(1, Ordering::Relaxed);
    }
}

#[lamellar::AmLocalData(Clone, Debug)]
struct LaunchAm {
    rand_index: Vec<usize>,
    counts: Darc<Vec<AtomicUsize>>,
}

#[lamellar::local_am]
impl LamellarAM for LaunchAm {
    async fn exec(self) {
        for idx in &self.rand_index {
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
    mut rand_index: Vec<usize>,
    counts: &Darc<Vec<AtomicUsize>>,
) -> Vec<impl Future<Output = ()>> {
    let slice_size = l_num_updates as f32 / launch_threads as f32;
    let mut launch_tasks = vec![];
    for tid in 0..launch_threads {
        let start = (tid as f32 * slice_size).round().round() as usize;
        let end = ((tid + 1) as f32 * slice_size).round() as usize;
        let split_index = rand_index.len() - (end - start);
        launch_tasks.push(world.exec_am_local(LaunchAm {
            rand_index: rand_index.split_off(split_index),
            counts: counts.clone(),
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

    if my_pe == 0 {
        cli.describe(num_pes);
    }

    let mut counts_data = Vec::with_capacity(local_count);
    for _ in 0..local_count {
        counts_data.push(AtomicUsize::new(0));
    }
    let counts = Darc::new(&world, counts_data).expect("unable to create darc");
    let mut rng: StdRng = SeedableRng::seed_from_u64(my_pe as u64);
    let rand_index = (0..l_num_updates)
        .into_iter()
        .map(|_| rng.gen_range(0, global_count))
        .collect::<Vec<usize>>();

    //create multiple launch tasks, that iterated through portions of rand_index in parallel
    // let launch_threads = match std::env::var("LAMELLAR_THREADS") {
    //     Ok(n) => n.parse::<usize>().unwrap(),
    //     Err(_) => 1,
    // };
    // let launch_threads = std::cmp::max(launch_threads / 2, 1);
    for _i in 0..iterations {
        world.barrier();
        let now = Instant::now();
        let launch_tasks = histo(
            l_num_updates,
            launch_threads,
            &world,
            rand_index.clone(),
            &counts,
        );

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

        println!(
            "pe {:?} sum {:?}",
            my_pe,
            counts
                .iter()
                .map(|e| e.load(Ordering::Relaxed))
                .sum::<usize>()
        );
        for elem in counts.iter() {
            elem.store(0, Ordering::SeqCst);
        }
    }
}
