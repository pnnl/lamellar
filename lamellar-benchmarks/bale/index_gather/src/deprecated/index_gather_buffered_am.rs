mod options;
use clap::Parser;

use lamellar::active_messaging::prelude::*;
use lamellar::array::prelude::*;
use lamellar::memregion::prelude::*;

use rand::prelude::*;
use std::future::Future;
use std::time::Instant;

//===== HISTO BEGIN ======

#[lamellar::AmData(Clone, Debug)]
struct IndexGatherBufferedAM {
    buff: std::vec::Vec<usize>,
    counts: ReadOnlyArray<usize>,
}

#[lamellar::am]
impl LamellarAM for IndexGatherBufferedAM {
    async fn exec(self) -> Vec<usize> {
        let counts_slice = self.counts.local_data();
        self.buff
            .iter()
            .map(|i| counts_slice[*i])
            .collect::<Vec<usize>>()
    }
}

fn buffered_ig<'a, I: Iterator<Item = &'a usize>>(
    num_pes: usize,
    buffer_size: usize,
    world: &LamellarWorld,
    rand_index: I,
    counts: &ReadOnlyArray<usize>,
) {
    let mut buffs: std::vec::Vec<std::vec::Vec<usize>> =
        vec![Vec::with_capacity(buffer_size); num_pes];
    let task_group = LamellarTaskGroup::new(world.clone());
    for idx in rand_index {
        let rank = idx % num_pes;
        let offset = idx / num_pes;

        buffs[rank].push(offset);
        if buffs[rank].len() >= buffer_size {
            let buff = buffs[rank].clone();
            task_group.exec_am_pe(
                rank,
                IndexGatherBufferedAM {
                    buff: buff,
                    counts: counts.clone(),
                },
            );
            buffs[rank].clear();
        }
    }

    //send any remaining buffered updates
    for rank in 0..num_pes {
        let buff = buffs[rank].clone();
        if buff.len() > 0 {
            task_group.exec_am_pe(
                rank,
                IndexGatherBufferedAM {
                    buff: buff,
                    counts: counts.clone(),
                },
            );
        }
    }
}

//===== HISTO END ======

fn main() {
    let world = lamellar::LamellarWorldBuilder::new().build();
    let my_pe = world.my_pe();
    let num_pes = world.num_pes();

    let cli = options::IndexGatherCli::parse();

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

    let counts = ReadOnlyArray::<usize>::new(
        world.clone(),
        global_count,
        lamellar::array::Distribution::Cyclic,
    );

    let rand_array = LocalLockArray::<usize>::new(
        world.clone(),
        g_num_updates,
        lamellar::array::Distribution::Block,
    );

    let now = Instant::now();
    let iter = rand_array
        .local_iter_mut()
        .chunks(l_num_updates / launch_threads)
        .for_each(move |slice| {
            let mut rng = thread_rng();
            // let mut rng = SmallRng::from_rng(&mut rng).unwrap();
            for e in slice {
                *e = rng.gen_range(0, global_count);
            }
        });
    world.block_on(iter);
    if my_pe == 0 {
        println!("{:?} init time {:?} ", my_pe, now.elapsed().as_secs_f64(),);
    }

    for _i in 0..iterations {
        world.barrier();
        let world_clone = world.clone();
        let counts_clone = counts.clone();
        let now = Instant::now();

        let iter = rand_array
            .local_iter()
            .chunks(l_num_updates / launch_threads)
            .for_each(move |slice| {
                buffered_ig(num_pes, buffer_size, &world_clone, slice, &counts_clone);
            });
        if my_pe == 0 {
            println!("{:?} issue time {:?} ", my_pe, now.elapsed().as_secs_f64(),);
        }
        world.block_on(iter);
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
                "{:?} global time {:?} MB {:?} MB/s: {:?} global mups: {:?} ",
                my_pe,
                global_time,
                world.MB_sent(),
                world.MB_sent() / global_time,
                ((l_num_updates * num_pes) as f64 / 1_000_000.0) / global_time
            );
        }
    }
}
