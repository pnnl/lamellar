#![allow(clippy::let_underscore_future)]

use lamellar::active_messaging::prelude::*;
use lamellar::darc::prelude::*;
use lamellar::memregion::prelude::*;

use crate::options::{HistoCli, IndexSize};

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

//===== HISTO BEGIN ======

//------ Safe AMs -----------
// Updates are atomic, indices are buffered u32s
#[lamellar::AmData(Clone, Debug)]
struct SafeU32 {
    index: u32,
    counts: Darc<Vec<AtomicUsize>>,
}

#[lamellar::am]
impl LamellarAM for SafeU32 {
    async fn exec(self) {
        self.counts[self.index as usize].fetch_add(1, Ordering::Relaxed);
    }
}

// Updates are not atomic, indices are buffered usizes
#[lamellar::AmData(Clone, Debug)]
struct SafeUsize {
    index: usize,
    counts: Darc<Vec<AtomicUsize>>,
}

#[lamellar::am]
impl LamellarAM for SafeUsize {
    async fn exec(self) {
        self.counts[self.index].fetch_add(1, Ordering::Relaxed);
    }
}

//-----------------------------------------------------------

//------ Unsafe AMs ------
// Updates are not atomic, indices are buffered u32s
#[lamellar::AmData(Clone, Debug)]
struct UnsafeU32 {
    index: u32,
    counts: SharedMemoryRegion<usize>,
}

#[lamellar::am]
impl LamellarAM for UnsafeU32 {
    async fn exec(self) {
        //this update would be unsafe and has potential for races / dropped updates
        unsafe { self.counts.as_mut_slice()[self.index as usize] += 1 };
    }
}

// Updates are not atomic, indices are buffered usizes
#[lamellar::AmData(Clone, Debug)]
struct UnsafeUsize {
    index: usize,
    counts: SharedMemoryRegion<usize>,
}

#[lamellar::am]
impl LamellarAM for UnsafeUsize {
    async fn exec(self) {
        unsafe { self.counts.as_mut_slice()[self.index] += 1 }; //this update would be unsafe and has potential for races / dropped updates
    }
}

//------------------------

// We likely want to issue updates from multiple threads to improve performance
// we can use a local Active Messages to do this.

enum AmType {
    SafeU32(Darc<Vec<AtomicUsize>>),
    SafeUsize(Darc<Vec<AtomicUsize>>),
    UnsafeU32(SharedMemoryRegion<usize>),
    UnsafeUsize(SharedMemoryRegion<usize>),
}

#[lamellar::AmLocalData(Clone, Debug)]
struct LaunchAmSafeU32 {
    rand_indices: Arc<Vec<usize>>,
    slice_start: usize,
    slice_end: usize,
    counts: Darc<Vec<AtomicUsize>>,
}

#[lamellar::local_am]
impl LamellarAM for LaunchAmSafeU32 {
    async fn exec(self) {
        for idx in &self.rand_indices[self.slice_start..self.slice_end] {
            let rank = idx % lamellar::num_pes;
            let index = idx / lamellar::num_pes;
            let _ = lamellar::world
                .exec_am_pe(
                    rank,
                    SafeU32 {
                        index: index as u32,
                        counts: self.counts.clone(),
                    },
                )
                .spawn(); //we could await here but we will just do a wait_all later instead
        }
    }
}

#[lamellar::AmLocalData(Clone, Debug)]
struct LaunchAmSafeUsize {
    rand_indices: Arc<Vec<usize>>,
    slice_start: usize,
    slice_end: usize,
    counts: Darc<Vec<AtomicUsize>>,
}

#[lamellar::local_am]
impl LamellarAM for LaunchAmSafeUsize {
    async fn exec(self) {
        for idx in &self.rand_indices[self.slice_start..self.slice_end] {
            let rank = idx % lamellar::num_pes;
            let index = idx / lamellar::num_pes;
            let _ = lamellar::world.spawn_am_pe(
                rank,
                SafeUsize {
                    index,
                    counts: self.counts.clone(),
                },
            ); //we could await here but we will just do a wait_all later instead
        }
    }
}

#[lamellar::AmLocalData(Clone, Debug)]
struct LaunchAmUnsafeU32 {
    rand_indices: Arc<Vec<usize>>,
    slice_start: usize,
    slice_end: usize,
    counts: SharedMemoryRegion<usize>,
}

#[lamellar::local_am]
impl LamellarAM for LaunchAmUnsafeU32 {
    async fn exec(self) {
        for idx in &self.rand_indices[self.slice_start..self.slice_end] {
            let rank = idx % lamellar::num_pes;
            let index = idx / lamellar::num_pes;
            let _ = lamellar::world.spawn_am_pe(
                rank,
                UnsafeU32 {
                    index: index as u32,
                    counts: self.counts.clone(),
                },
            ); //we could await here but we will just do a wait_all later instead
        }
    }
}

#[lamellar::AmLocalData(Clone, Debug)]
struct LaunchAmUnsafeUsize {
    rand_indices: Arc<Vec<usize>>,
    slice_start: usize,
    slice_end: usize,
    counts: SharedMemoryRegion<usize>,
}

#[lamellar::local_am]
impl LamellarAM for LaunchAmUnsafeUsize {
    async fn exec(self) {
        for idx in &self.rand_indices[self.slice_start..self.slice_end] {
            let rank = idx % lamellar::num_pes;
            let index = idx / lamellar::num_pes;
            let _ = lamellar::world.spawn_am_pe(
                rank,
                UnsafeUsize {
                    index,
                    counts: self.counts.clone(),
                },
            ); //we could await here but we will just do a wait_all later instead
        }
    }
}

fn launch_ams(
    world: &LamellarWorld,
    histo_config: &HistoCli,
    rand_indices: &Arc<Vec<usize>>,
    am_type: AmType,
) -> Pin<Box<dyn Future<Output = Vec<()>>>> {
    let num_pes = world.num_pes();
    let slice_size = histo_config.pe_updates(num_pes) as f32 / histo_config.launch_threads as f32;
    let mut launch_tasks = vec![];

    for tid in 0..histo_config.launch_threads {
        let start = (tid as f32 * slice_size).round() as usize;
        let end = (tid as f32 * slice_size + slice_size).round() as usize;
        launch_tasks.push(match am_type {
            AmType::SafeU32(ref counts) => world.spawn_am_local(LaunchAmSafeU32 {
                rand_indices: rand_indices.clone(),
                slice_start: start,
                slice_end: end,
                counts: counts.clone(),
            }),
            AmType::SafeUsize(ref counts) => world.spawn_am_local(LaunchAmSafeUsize {
                rand_indices: rand_indices.clone(),
                slice_start: start,
                slice_end: end,
                counts: counts.clone(),
            }),
            AmType::UnsafeU32(ref counts) => world.spawn_am_local(LaunchAmUnsafeU32 {
                rand_indices: rand_indices.clone(),
                slice_start: start,
                slice_end: end,
                counts: counts.clone(),
            }),
            AmType::UnsafeUsize(ref counts) => world.spawn_am_local(LaunchAmUnsafeUsize {
                rand_indices: rand_indices.clone(),
                slice_start: start,
                slice_end: end,
                counts: counts.clone(),
            }),
        });
    }
    Box::pin(futures::future::join_all(launch_tasks))
}

pub fn histo(
    world: &lamellar::LamellarWorld,
    histo_config: &HistoCli,
    rand_indices: &Arc<Vec<usize>>,
    safe: bool,
    index_size: &IndexSize,
) -> (Duration, Duration, Duration, Duration) {
    let num_pes = world.num_pes();
    world.barrier();
    let mut timer = Instant::now();
    let (_init_time, launch_tasks) = if safe {
        let mut counts_inner = Vec::with_capacity(histo_config.pe_table_size(num_pes));
        for _ in 0..histo_config.pe_table_size(num_pes) {
            counts_inner.push(AtomicUsize::new(0));
        }
        let counts = Darc::new(world, counts_inner)
            .block()
            .expect("darc should be created");
        world.barrier();
        let init_time = timer.elapsed();
        timer = Instant::now();
        let launch_tasks = match index_size {
            IndexSize::U32 => {
                launch_ams(world, histo_config, rand_indices, AmType::SafeU32(counts))
            }
            IndexSize::Usize => {
                launch_ams(world, histo_config, rand_indices, AmType::SafeUsize(counts))
            }
        };
        (init_time, launch_tasks)
    } else {
        let counts = world
            .alloc_shared_mem_region(histo_config.pe_table_size(num_pes))
            .block();
        unsafe {
            for elem in counts.as_mut_slice().iter_mut() {
                *elem = 0;
            }
        }
        world.barrier();
        let init_time = timer.elapsed();
        timer = Instant::now();
        let launch_tasks = match index_size {
            IndexSize::U32 => {
                launch_ams(world, histo_config, rand_indices, AmType::UnsafeU32(counts))
            }
            IndexSize::Usize => launch_ams(
                world,
                histo_config,
                rand_indices,
                AmType::UnsafeUsize(counts),
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
