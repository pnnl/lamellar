#![allow(clippy::let_underscore_future)]

use lamellar::active_messaging::prelude::*;
use lamellar::array::prelude::*;
use lamellar::darc::prelude::*;

use crate::options::{IndexSize, RandPermCli};

use rand::prelude::*;

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

//------ Safe AMs -----------
// Updates are atomic, indices are buffered u32s
#[lamellar::AmData]
struct CasDartU32Am {
    target: Darc<Vec<AtomicUsize>>,
    dart_index: u32,
    val: usize,
}

#[lamellar::am]
impl LamellarAM for CasDartU32Am {
    async fn exec(self) {
        if self.target[self.dart_index as usize]
            .compare_exchange(usize::MAX, self.val, Ordering::Relaxed, Ordering::Relaxed)
            .is_err()
        {
            // if the slot is consumed, pick a new location to launch this dart!
            let dart_index = {
                let mut thread_rng = thread_rng();
                let mut rng = SmallRng::from_rng(&mut thread_rng).unwrap();
                rng.gen_range(0, self.target.len() * lamellar::num_pes)
            }; // we scope this because thread_rng is not send, so we need to ensure it drops before the call to await below

            let pe_index = dart_index / lamellar::num_pes;
            let pe = dart_index % lamellar::num_pes;
            let _ = lamellar::world
                .exec_am_pe(
                    pe,
                    CasDartU32Am {
                        target: self.target.clone(),
                        val: self.val,
                        dart_index: pe_index as u32,
                    },
                )
                .spawn();
            // .await; //awaiting here prevents the original AM from returning until the dart has landed somewhere
        }
    }
}

#[lamellar::AmData]
struct CasDartUsizeAm {
    target: Darc<Vec<AtomicUsize>>,
    dart_index: usize,
    val: usize,
}

#[lamellar::am]
impl LamellarAM for CasDartUsizeAm {
    async fn exec(self) {
        if self.target[self.dart_index]
            .compare_exchange(usize::MAX, self.val, Ordering::Relaxed, Ordering::Relaxed)
            .is_err()
        {
            // if the slot is consumed, pick a new location to launch this dart!
            let dart_index = {
                let mut thread_rng = thread_rng();
                let mut rng = SmallRng::from_rng(&mut thread_rng).unwrap();
                rng.gen_range(0, self.target.len() * lamellar::num_pes)
            }; // we scope this because thread_rng is not send, so we need to ensure it drops before the call to await below

            let pe_index = dart_index / lamellar::num_pes;
            let pe = dart_index % lamellar::num_pes;
            let _ = lamellar::world
                .exec_am_pe(
                    pe,
                    CasDartUsizeAm {
                        target: self.target.clone(),
                        val: self.val,
                        dart_index: pe_index,
                    },
                )
                .spawn();
            // .await; //awaiting here prevents the original AM from returning until the dart has landed somewhere
        }
    }
}

//--------------------------

// We likely want to issue updates from multiple threads to improve performance
// we can use a local Active Messages to do this.

#[derive(Clone, Debug)]
enum AmType {
    CasDartU32Am(Darc<Vec<AtomicUsize>>),
    CasDartUsizeAm(Darc<Vec<AtomicUsize>>),
}
impl AmType {
    fn len(&self) -> usize {
        match self {
            AmType::CasDartU32Am(target) | AmType::CasDartUsizeAm(target) => target.len(),
        }
    }
}
#[lamellar::AmLocalData(Clone, Debug)]
struct LaunchAm {
    val_start: usize,
    val_end: usize,
    target: AmType,
}

#[lamellar::local_am]
impl LamellarAM for LaunchAm {
    async fn exec(self) {
        let mut thread_rng = thread_rng();
        let mut rng = SmallRng::from_rng(&mut thread_rng).unwrap();
        let target_len = self.target.len();
        for val in self.val_start..self.val_end {
            let dart_index = rng.gen_range(0, target_len * lamellar::num_pes);
            let pe_index = dart_index / lamellar::num_pes;
            let pe = dart_index % lamellar::num_pes;

            let _ = match &self.target {
                AmType::CasDartU32Am(target) => lamellar::world
                    .exec_am_pe(
                        pe,
                        CasDartU32Am {
                            val,
                            target: target.clone(),
                            dart_index: pe_index as u32,
                        },
                    )
                    .spawn(),
                AmType::CasDartUsizeAm(target) => lamellar::world
                    .exec_am_pe(
                        pe,
                        CasDartUsizeAm {
                            val,
                            target: target.clone(),
                            dart_index: pe_index,
                        },
                    )
                    .spawn(),
            };
        }
    }
}

fn launch_ams(
    world: &LamellarWorld,
    rand_perm_config: &RandPermCli,
    am_type: AmType,
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
                    target: am_type.clone(),
                })
                .spawn(),
        );
    }
    Box::pin(futures::future::join_all(launch_tasks))
}

pub fn rand_perm(
    world: &lamellar::LamellarWorld,
    rand_perm_config: &RandPermCli,
    index_size: &IndexSize,
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
    let target = Darc::new(world, target_inner)
        .block()
        .expect("darc should be created");
    world.barrier();
    let _init_time = timer.elapsed();
    timer = Instant::now();
    let launch_tasks = match index_size {
        IndexSize::U32 => launch_ams(
            world,
            rand_perm_config,
            AmType::CasDartU32Am(target.clone()),
        ),
        IndexSize::Usize => launch_ams(
            world,
            rand_perm_config,
            AmType::CasDartUsizeAm(target.clone()),
        ),
        _ => unreachable!(),
    };
    world.block_on(launch_tasks);
    world.wait_all();
    let target = target.into_localrw().block(); //a cheap hack to ensure all other references to the darc are dropped, and thus the all the launched active messages have completed
    world.barrier();
    let perm_time = timer.elapsed();

    let collect_timer = Instant::now();
    let data = target
        .read()
        .block()
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
        .spawn();
    world.wait_all();
    world.barrier();

    (
        perm_time,
        collect_time,
        global_finish_time,
        sum.load(Ordering::Relaxed),
    )
}
