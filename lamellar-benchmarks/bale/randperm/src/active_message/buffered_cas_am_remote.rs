#![allow(clippy::let_underscore_future)]

use lamellar::active_messaging::prelude::*;
use lamellar::array::prelude::*;
use lamellar::darc::prelude::*;

use crate::options::RandPermCli;

use rand::prelude::*;

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

//------ Safe AMs -----------
// Updates are atomic, indices are buffered u32s
#[lamellar::AmData]
struct BufferedCasDartAm {
    target: Darc<(Vec<AtomicUsize>, AtomicUsize)>,
    darts: Vec<usize>,
    buffer_size: usize,
}

#[lamellar::am]
impl LamellarAM for BufferedCasDartAm {
    async fn exec(self) {
        let mut thread_rng = thread_rng();
        let mut rng = SmallRng::from_rng(&mut thread_rng).unwrap();
        let mut failed_darts = vec![Vec::with_capacity(self.buffer_size); lamellar::num_pes];
        for val in &self.darts {
            let mut res: Result<usize, usize> = Err(0); // incase this pe is already full.
            while res.is_err() && self.target.1.load(Ordering::Relaxed) < self.target.0.len() {
                let index = rng.gen_range(0, self.target.0.len());
                res = self.target.0[index].compare_exchange(
                    usize::MAX,
                    *val,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                );
            }
            if res.is_err() {
                // if the slot is consumed, pick a new location to launch this dart!
                let pe = rng.gen_range(0, lamellar::num_pes);
                failed_darts[pe].push(*val);
                if failed_darts[pe].len() >= self.buffer_size {
                    let mut darts = Vec::with_capacity(self.buffer_size);
                    std::mem::swap(&mut failed_darts[pe], &mut darts);
                    let _ = lamellar::world
                        .exec_am_pe(
                            pe,
                            BufferedCasDartAm {
                                target: self.target.clone(),
                                darts,
                                buffer_size: self.buffer_size,
                            },
                        )
                        .spawn(); //we could await here but we will just do a wait_all later instead
                }
            } else {
                self.target.1.fetch_add(1, Ordering::Relaxed);
            }
        }
        for (pe, darts) in failed_darts.drain(..).enumerate() {
            if !darts.is_empty() {
                let _ = lamellar::world
                    .exec_am_pe(
                        pe,
                        BufferedCasDartAm {
                            target: self.target.clone(),
                            darts,
                            buffer_size: self.buffer_size,
                        },
                    )
                    .spawn(); //we could await here but we will just do a wait_all later instead
            }
        }
    }
}

#[lamellar::AmLocalData(Clone, Debug)]
struct LaunchAm {
    val_start: usize,
    val_end: usize,
    target: Darc<(Vec<AtomicUsize>, AtomicUsize)>,
    buffer_size: usize,
}

#[lamellar::local_am]
impl LamellarAM for LaunchAm {
    async fn exec(self) {
        let mut thread_rng = thread_rng();
        let mut rng = SmallRng::from_rng(&mut thread_rng).unwrap();
        let mut buffered_darts = vec![Vec::new(); lamellar::num_pes];
        for val in self.val_start..self.val_end {
            let pe = rng.gen_range(0, lamellar::num_pes);
            buffered_darts[pe].push(val);
            if buffered_darts[pe].len() >= self.buffer_size {
                let mut darts = Vec::with_capacity(self.buffer_size);
                std::mem::swap(&mut buffered_darts[pe], &mut darts);
                let _ = lamellar::world
                    .exec_am_pe(
                        pe,
                        BufferedCasDartAm {
                            target: self.target.clone(),
                            darts,
                            buffer_size: self.buffer_size,
                        },
                    )
                    .spawn(); //we could await here but we will just do a wait_all later instead
            }
        }

        for (pe, darts) in buffered_darts.drain(..).enumerate() {
            if !darts.is_empty() {
                let _ = lamellar::world
                    .exec_am_pe(
                        pe,
                        BufferedCasDartAm {
                            target: self.target.clone(),
                            darts,
                            buffer_size: self.buffer_size,
                        },
                    )
                    .spawn(); //we could await here but we will just do a wait_all later instead
            }
        }
    }
}

fn launch_ams(
    world: &LamellarWorld,
    rand_perm_config: &RandPermCli,
    target: Darc<(Vec<AtomicUsize>, AtomicUsize)>,
) -> Pin<Box<dyn Future<Output = Vec<()>>>> {
    let num_pes = world.num_pes();
    let my_pe = world.my_pe();
    let slice_size =
        rand_perm_config.pe_table_size(num_pes) as f32 / rand_perm_config.launch_threads as f32;
    let mut launch_tasks = vec![];

    for tid in 0..rand_perm_config.launch_threads {
        let start = (tid as f32 * slice_size).round() as usize
            + (my_pe * rand_perm_config.pe_table_size(num_pes));
        let end = (tid as f32 * slice_size + slice_size).round() as usize
            + (my_pe * rand_perm_config.pe_table_size(num_pes));

        launch_tasks.push(
            world
                .exec_am_local(LaunchAm {
                    val_start: start,
                    val_end: end,
                    target: target.clone(),
                    buffer_size: rand_perm_config.buffer_size,
                })
                .spawn(),
        );
    }
    Box::pin(futures::future::join_all(launch_tasks))
}

pub fn rand_perm(
    world: &lamellar::LamellarWorld,
    rand_perm_config: &RandPermCli,
) -> (Duration, Duration, Duration, usize) {
    let num_pes = world.num_pes();
    let local_lens =
        AtomicArray::new(world, world.num_pes(), lamellar::Distribution::Block).block();
    let the_array = LocalRwDarc::new(world, vec![0; rand_perm_config.pe_table_size(num_pes)])
        .block()
        .unwrap();
    world.barrier();

    let mut timer = Instant::now();
    let target_size = rand_perm_config.pe_table_size(num_pes) * rand_perm_config.target_factor;
    let mut target_inner = Vec::with_capacity(target_size);
    for _ in 0..target_size {
        target_inner.push(AtomicUsize::new(usize::MAX));
    }
    let target = Darc::new(world, (target_inner, AtomicUsize::new(0)))
        .block()
        .expect("darc should be created");
    world.barrier();
    let _init_time = timer.elapsed();
    timer = Instant::now();
    let launch_tasks = launch_ams(world, rand_perm_config, target.clone());
    world.block_on(launch_tasks);
    world.wait_all();
    world.barrier();
    let perm_time = timer.elapsed();

    let collect_timer = Instant::now();
    let data = target
        .0
        .iter()
        .map(|x| x.load(Ordering::Relaxed))
        .filter(|x| *x != usize::MAX)
        .collect::<Vec<_>>();
    super::collect_perm(world, rand_perm_config, data, &the_array, &local_lens);
    let collect_time = collect_timer.elapsed();

    let global_finish_time = timer.elapsed();

    let sum = Darc::new(world, AtomicUsize::new(0))
        .block()
        .expect("darc should be created");
    let local_sum = the_array.read().block().iter().sum::<usize>();

    let _ = world
        .exec_am_pe(
            0,
            super::SumAm {
                sum: sum.clone(),
                amt: local_sum,
            },
        )
        .spawn(); //we could await here but we will just do a wait_all later instead
    world.wait_all();
    world.barrier();
    // if my_pe == 0 {
    //     println!(
    //         "[{:?}]: target_pe_sum: {:?}",
    //         world.my_pe(),
    //         sum.load(Ordering::Relaxed)
    //     );
    // }

    (
        perm_time,
        collect_time,
        global_finish_time,
        sum.load(Ordering::Relaxed),
    )
}
