// clippy misattributes let_and_return warnings to macro-expanded #[lamellar::am] bodies
#![allow(clippy::let_and_return)]

use lamellar::active_messaging::prelude::*;
use lamellar::darc::prelude::*;
use lamellar::memregion::prelude::*;

use crate::options::{IndexGatherCli, IndexSize};

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};

//===== Index Gather BEGIN ======

//------ Safe AMs -----------
// Updates are atomic, indices are buffered u32s
#[lamellar::AmData(Clone, Debug)]
struct SafeU32 {
    index: u32,
    table: Darc<Vec<usize>>,
}

#[lamellar::am]
impl LamellarAM for SafeU32 {
    async fn exec(self) -> usize {
        self.table[self.index as usize]
    }
}

// Updates are not atomic, indices are buffered usizes
#[lamellar::AmData(Clone, Debug)]
struct SafeUsize {
    index: usize,
    table: Darc<Vec<usize>>,
}

#[lamellar::am]
impl LamellarAM for SafeUsize {
    async fn exec(self) -> usize {
        self.table[self.index]
    }
}

//-----------------------------------------------------------

//------ Unsafe AMs ------
// Updates are not atomic, indices are buffered u32s
#[lamellar::AmData(Clone, Debug)]
struct UnsafeU32 {
    index: u32,
    table: SharedMemoryRegion<usize>,
}

#[lamellar::am]
impl LamellarAM for UnsafeU32 {
    async fn exec(self) -> usize {
        unsafe { self.table.as_mut_slice()[self.index as usize] }
    }
}

// Updates are not atomic, indices are buffered usizes
#[lamellar::AmData(Clone, Debug)]
struct UnsafeUsize {
    index: usize,
    table: SharedMemoryRegion<usize>,
}

#[lamellar::am]
impl LamellarAM for UnsafeUsize {
    async fn exec(self) -> usize {
        unsafe { self.table.as_mut_slice()[self.index] } //this update would be unsafe and has potential for races / dropped updates
    }
}

//------------------------

// We likely want to issue updates from multiple threads to improve performance
// we can use a local Active Messages to do this.
#[derive(Clone, Debug)]
enum AmType {
    SafeU32(Darc<Vec<usize>>),
    SafeUsize(Darc<Vec<usize>>),
    UnsafeU32(SharedMemoryRegion<usize>),
    UnsafeUsize(SharedMemoryRegion<usize>),
}

#[lamellar::AmLocalData(Clone, Debug)]
struct LaunchAm {
    rand_indices: Arc<Vec<usize>>,
    slice_start: usize,
    slice_end: usize,
    table: AmType,
}

#[lamellar::local_am]
impl LamellarAM for LaunchAm {
    async fn exec(self) -> Vec<usize> {
        let mut reqs = vec![];
        for idx in &self.rand_indices[self.slice_start..self.slice_end] {
            let rank = idx % lamellar::num_pes;
            let index = idx / lamellar::num_pes;
            reqs.push(match &self.table {
                AmType::SafeU32(target) => lamellar::world
                    .exec_am_pe(
                        rank,
                        SafeU32 {
                            index: index as u32,
                            table: target.clone(),
                        },
                    )
                    .spawn(),
                AmType::SafeUsize(target) => lamellar::world
                    .exec_am_pe(
                        rank,
                        SafeUsize {
                            index,
                            table: target.clone(),
                        },
                    )
                    .spawn(),
                AmType::UnsafeU32(target) => lamellar::world
                    .exec_am_pe(
                        rank,
                        UnsafeU32 {
                            index: index as u32,
                            table: target.clone(),
                        },
                    )
                    .spawn(),
                AmType::UnsafeUsize(target) => lamellar::world
                    .exec_am_pe(
                        rank,
                        UnsafeUsize {
                            index,
                            table: target.clone(),
                        },
                    )
                    .spawn(),
            });
        }
        futures::future::join_all(reqs).await
    }
}

fn launch_ams(
    world: &LamellarWorld,
    ig_config: &IndexGatherCli,
    rand_indices: &Arc<Vec<usize>>,
    am_type: AmType,
) -> Pin<Box<dyn Future<Output = Vec<Vec<usize>>>>> {
    let num_pes = world.num_pes();
    let slice_size = ig_config.pe_updates(num_pes) as f32 / ig_config.launch_threads as f32;
    let mut launch_tasks = vec![];

    for tid in 0..ig_config.launch_threads {
        let start = (tid as f32 * slice_size).round() as usize;
        let end = (tid as f32 * slice_size + slice_size).round() as usize;
        launch_tasks.push(
            world
                .exec_am_local(LaunchAm {
                    rand_indices: rand_indices.clone(),
                    slice_start: start,
                    slice_end: end,
                    table: am_type.clone(),
                })
                .spawn(),
        );
    }
    Box::pin(futures::future::join_all(launch_tasks))
}

pub fn index_gather(
    world: &lamellar::LamellarWorld,
    ig_config: &IndexGatherCli,
    rand_indices: &Arc<Vec<usize>>,
    safe: bool,
    index_size: &IndexSize,
) -> (Duration, Duration, Duration, Duration) {
    let num_pes = world.num_pes();
    let my_pe = world.my_pe();
    world.barrier();
    let mut timer = Instant::now();
    let (_init_time, launch_tasks) = if safe {
        let mut table_inner = Vec::with_capacity(ig_config.pe_table_size(num_pes));
        for i in 0..ig_config.pe_table_size(num_pes) {
            table_inner.push(my_pe * ig_config.pe_table_size(num_pes) + i);
        }
        let table = Darc::new(world, table_inner)
            .block()
            .expect("darc should be created");
        world.barrier();
        let init_time = timer.elapsed();
        timer = Instant::now();
        let launch_tasks = match index_size {
            IndexSize::U32 => launch_ams(world, ig_config, rand_indices, AmType::SafeU32(table)),
            IndexSize::Usize => {
                launch_ams(world, ig_config, rand_indices, AmType::SafeUsize(table))
            }
        };
        (init_time, launch_tasks)
    } else {
        let table = world
            .alloc_shared_mem_region(ig_config.pe_table_size(num_pes))
            .block();
        unsafe {
            for elem in table.as_mut_slice().iter_mut() {
                *elem = 0;
            }
        }
        world.barrier();
        let init_time = timer.elapsed();
        timer = Instant::now();
        let launch_tasks = match index_size {
            IndexSize::U32 => launch_ams(world, ig_config, rand_indices, AmType::UnsafeU32(table)),
            IndexSize::Usize => {
                launch_ams(world, ig_config, rand_indices, AmType::UnsafeUsize(table))
            }
        };
        (init_time, launch_tasks)
    };

    let launch_issue_time = timer.elapsed();
    world.block_on(launch_tasks);
    let launch_finish_time = timer.elapsed();
    world.wait_all();
    let local_finish_time = timer.elapsed();
    world.barrier();
    let global_finish_time = timer.elapsed();
    (
        launch_issue_time,
        launch_finish_time,
        local_finish_time,
        global_finish_time,
    )
}
