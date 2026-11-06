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

//===== HISTO BEGIN ======

//------ Safe AMs -----------
// Updates are atomic, indices are buffered u32s
#[lamellar::AmData(Clone, Debug, AmGroup)]
struct SafeU32Group {
    index: u32,
    #[AmGroup(static)]
    table: Darc<Vec<usize>>,
}

#[lamellar::am(AmGroup)]
impl LamellarAM for SafeU32Group {
    async fn exec(self) -> usize {
        self.table[self.index as usize]
    }
}

// Updates are not atomic, indices are buffered usizes
#[lamellar::AmData(Clone, Debug, AmGroup)]
struct SafeUsizeGroup {
    index: usize,
    #[AmGroup(static)]
    table: Darc<Vec<usize>>,
}

#[lamellar::am(AmGroup)]
impl LamellarAM for SafeUsizeGroup {
    async fn exec(self) -> usize {
        self.table[self.index]
    }
}

//-----------------------------------------------------------

//------ Unsafe AMs ------
// Updates are not atomic, indices are buffered u32s
#[lamellar::AmData(Clone, Debug, AmGroup)]
struct UnsafeU32Group {
    index: u32,
    #[AmGroup(static)]
    table: SharedMemoryRegion<usize>,
}

#[lamellar::am(AmGroup)]
impl LamellarAM for UnsafeU32Group {
    async fn exec(self) -> usize {
        //this update would be unsafe and has potential for races / dropped updates
        unsafe { self.table.as_mut_slice()[self.index as usize] }
    }
}

// Updates are not atomic, indices are buffered usizes
#[lamellar::AmData(Clone, Debug, AmGroup)]
struct UnsafeUsizeGroup {
    index: usize,
    #[AmGroup(static)]
    table: SharedMemoryRegion<usize>,
}

#[lamellar::am(AmGroup)]
impl LamellarAM for UnsafeUsizeGroup {
    async fn exec(self) -> usize {
        unsafe { self.table.as_mut_slice()[self.index] } //this update would be Unsafe and has potential for races / dropped updates
    }
}

//------------------------

// We likely want to issue updates from multiple threads to improve performance
// we can use a local Active Messages to do this.

// "Group" postfix distinguishes these from the non-grouped variants in am.rs
#[allow(clippy::enum_variant_names)]
enum AmType {
    SafeU32Group(Darc<Vec<usize>>),
    SafeUsizeGroup(Darc<Vec<usize>>),
    UnsafeU32Group(SharedMemoryRegion<usize>),
    UnsafeUsizeGroup(SharedMemoryRegion<usize>),
}

// unforutunately we need unique launch am when using AMGroups because the typed_am_group is unique to each AM type
#[lamellar::AmLocalData(Clone, Debug)]
struct LaunchAmSafeU32Group {
    rand_indices: Arc<Vec<usize>>,
    slice_start: usize,
    slice_end: usize,
    table: Darc<Vec<usize>>,
}

#[lamellar::local_am]
impl LamellarAM for LaunchAmSafeU32Group {
    async fn exec(self) -> Vec<usize> {
        let mut tg = typed_am_group!(SafeU32Group, lamellar::world.clone());
        for idx in &self.rand_indices[self.slice_start..self.slice_end] {
            let rank = idx % lamellar::num_pes;
            let index = idx / lamellar::num_pes;
            tg.add_am_pe(
                rank,
                SafeU32Group {
                    index: index as u32,
                    table: self.table.clone(),
                },
            );
        }
        let res = tg.exec().await;
        // println!("returned taskgroup! {:?}", res.len());
        res.iter()
            .map(|x| match x {
                AmGroupResult::Pe(_, v) => *v,
                _ => panic!("invalid result"),
            })
            .collect::<Vec<_>>()
        // Vec::<usize>::new()
    }
}

#[lamellar::AmLocalData(Clone, Debug)]
struct LaunchAmSafeUsizeGroup {
    rand_indices: Arc<Vec<usize>>,
    slice_start: usize,
    slice_end: usize,
    table: Darc<Vec<usize>>,
}

#[lamellar::local_am]
impl LamellarAM for LaunchAmSafeUsizeGroup {
    async fn exec(self) -> Vec<usize> {
        let mut tg = typed_am_group!(SafeUsizeGroup, lamellar::world.clone());
        for idx in &self.rand_indices[self.slice_start..self.slice_end] {
            let rank = idx % lamellar::num_pes;
            let index = idx / lamellar::num_pes;
            tg.add_am_pe(
                rank,
                SafeUsizeGroup {
                    index,
                    table: self.table.clone(),
                },
            );
        }
        tg.exec()
            .await
            .iter()
            .map(|x| match x {
                AmGroupResult::Pe(_, v) => *v,
                _ => panic!("invalid result"),
            })
            .collect::<Vec<_>>()
    }
}

#[lamellar::AmLocalData(Clone, Debug)]
struct LaunchAmUnsafeU32Group {
    rand_indices: Arc<Vec<usize>>,
    slice_start: usize,
    slice_end: usize,
    table: SharedMemoryRegion<usize>,
}

#[lamellar::local_am]
impl LamellarAM for LaunchAmUnsafeU32Group {
    async fn exec(self) -> Vec<usize> {
        let mut tg = typed_am_group!(UnsafeU32Group, lamellar::world.clone());
        for idx in &self.rand_indices[self.slice_start..self.slice_end] {
            let rank = idx % lamellar::num_pes;
            let index = idx / lamellar::num_pes;
            tg.add_am_pe(
                rank,
                UnsafeU32Group {
                    index: index as u32,
                    table: self.table.clone(),
                },
            );
        }
        tg.exec()
            .await
            .iter()
            .map(|x| match x {
                AmGroupResult::Pe(_, v) => *v,
                _ => panic!("invalid result"),
            })
            .collect::<Vec<_>>()
    }
}

#[lamellar::AmLocalData(Clone, Debug)]
struct LaunchAmUnsafeUsizeGroup {
    rand_indices: Arc<Vec<usize>>,
    slice_start: usize,
    slice_end: usize,
    table: SharedMemoryRegion<usize>,
}

#[lamellar::local_am]
impl LamellarAM for LaunchAmUnsafeUsizeGroup {
    async fn exec(self) -> Vec<usize> {
        let mut tg = typed_am_group!(UnsafeUsizeGroup, lamellar::world.clone());
        for idx in &self.rand_indices[self.slice_start..self.slice_end] {
            let rank = idx % lamellar::num_pes;
            let index = idx / lamellar::num_pes;
            tg.add_am_pe(
                rank,
                UnsafeUsizeGroup {
                    index,
                    table: self.table.clone(),
                },
            );
        }
        tg.exec()
            .await
            .iter()
            .map(|x| match x {
                AmGroupResult::Pe(_, v) => *v,
                _ => panic!("invalid result"),
            })
            .collect::<Vec<_>>()
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
        launch_tasks.push(match am_type {
            AmType::SafeU32Group(ref table) => world
                .exec_am_local(LaunchAmSafeU32Group {
                    rand_indices: rand_indices.clone(),
                    slice_start: start,
                    slice_end: end,
                    table: table.clone(),
                })
                .spawn(),
            AmType::SafeUsizeGroup(ref table) => world
                .exec_am_local(LaunchAmSafeUsizeGroup {
                    rand_indices: rand_indices.clone(),
                    slice_start: start,
                    slice_end: end,
                    table: table.clone(),
                })
                .spawn(),
            AmType::UnsafeU32Group(ref table) => world
                .exec_am_local(LaunchAmUnsafeU32Group {
                    rand_indices: rand_indices.clone(),
                    slice_start: start,
                    slice_end: end,
                    table: table.clone(),
                })
                .spawn(),
            AmType::UnsafeUsizeGroup(ref table) => world
                .exec_am_local(LaunchAmUnsafeUsizeGroup {
                    rand_indices: rand_indices.clone(),
                    slice_start: start,
                    slice_end: end,
                    table: table.clone(),
                })
                .spawn(),
        });
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
    std::env::set_var(
        "LAMELLAR_BATCH_OP_SIZE",
        format!("{}", ig_config.buffer_size),
    );
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
            IndexSize::U32 => {
                launch_ams(world, ig_config, rand_indices, AmType::SafeU32Group(table))
            }
            IndexSize::Usize => launch_ams(
                world,
                ig_config,
                rand_indices,
                AmType::SafeUsizeGroup(table),
            ),
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
            IndexSize::U32 => launch_ams(
                world,
                ig_config,
                rand_indices,
                AmType::UnsafeU32Group(table),
            ),
            IndexSize::Usize => launch_ams(
                world,
                ig_config,
                rand_indices,
                AmType::UnsafeUsizeGroup(table),
            ),
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
