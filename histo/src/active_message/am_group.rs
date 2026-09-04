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
#[lamellar::AmData(Clone, Debug, AmGroup)]
struct SafeU32Group {
    index: u32,
    #[AmGroup(static)]
    counts: Darc<Vec<AtomicUsize>>,
}

#[lamellar::am(AmGroup)]
impl LamellarAM for SafeU32Group {
    async fn exec(self) {
        self.counts[self.index as usize].fetch_add(1, Ordering::Relaxed);
    }
}

// Updates are not atomic, indices are buffered usizes
#[lamellar::AmData(Clone, Debug, AmGroup)]
struct SafeUsizeGroup {
    index: usize,
    #[AmGroup(static)]
    counts: Darc<Vec<AtomicUsize>>,
}

#[lamellar::am(AmGroup)]
impl LamellarAM for SafeUsizeGroup {
    async fn exec(self) {
        self.counts[self.index].fetch_add(1, Ordering::Relaxed);
    }
}

//-----------------------------------------------------------

//------ Unsafe AMs ------
// Updates are not atomic, indices are buffered u32s
#[lamellar::AmData(Clone, Debug, AmGroup)]
struct UnsafeU32Group {
    index: u32,
    #[AmGroup(static)]
    counts: SharedMemoryRegion<usize>,
}

#[lamellar::am(AmGroup)]
impl LamellarAM for UnsafeU32Group {
    async fn exec(self) {
        //this update would be unsafe and has potential for races / dropped updates
        unsafe { self.counts.as_mut_slice().unwrap()[self.index as usize] += 1 };
    }
}

// Updates are not atomic, indices are buffered usizes
#[lamellar::AmData(Clone, Debug, AmGroup)]
struct UnsafeUsizeGroup {
    index: usize,
    #[AmGroup(static)]
    counts: SharedMemoryRegion<usize>,
}

#[lamellar::am(AmGroup)]
impl LamellarAM for UnsafeUsizeGroup {
    async fn exec(self) {
        unsafe { self.counts.as_mut_slice().unwrap()[self.index] += 1 }; //this update would be Unsafe and has potential for races / dropped updates
    }
}

//------------------------

// We likely want to issue updates from multiple threads to improve performance
// we can use a local Active Messages to do this.

// "Group" postfix distinguishes these from the non-grouped variants in am.rs
#[allow(clippy::enum_variant_names)]
enum AmType {
    SafeU32Group(Darc<Vec<AtomicUsize>>),
    SafeUsizeGroup(Darc<Vec<AtomicUsize>>),
    UnsafeU32Group(SharedMemoryRegion<usize>),
    UnsafeUsizeGroup(SharedMemoryRegion<usize>),
}

#[lamellar::AmLocalData(Clone, Debug)]
struct LaunchAmSafeU32Group {
    rand_indices: Arc<Vec<usize>>,
    slice_start: usize,
    slice_end: usize,
    counts: Darc<Vec<AtomicUsize>>,
}

#[lamellar::local_am]
impl LamellarAM for LaunchAmSafeU32Group {
    async fn exec(self) {
        let mut tg = typed_am_group!(SafeU32Group, lamellar::world.clone());
        for idx in &self.rand_indices[self.slice_start..self.slice_end] {
            let rank = idx % lamellar::num_pes;
            let index = idx / lamellar::num_pes;
            tg.add_am_pe(
                rank,
                SafeU32Group {
                    index: index as u32,
                    counts: self.counts.clone(),
                },
            );
        }
        tg.exec().await;
    }
}

#[lamellar::AmLocalData(Clone, Debug)]
struct LaunchAmSafeUsizeGroup {
    rand_indices: Arc<Vec<usize>>,
    slice_start: usize,
    slice_end: usize,
    counts: Darc<Vec<AtomicUsize>>,
}

#[lamellar::local_am]
impl LamellarAM for LaunchAmSafeUsizeGroup {
    async fn exec(self) {
        let mut tg = typed_am_group!(SafeUsizeGroup, lamellar::world.clone());
        for idx in &self.rand_indices[self.slice_start..self.slice_end] {
            let rank = idx % lamellar::num_pes;
            let index = idx / lamellar::num_pes;
            tg.add_am_pe(
                rank,
                SafeUsizeGroup {
                    index,
                    counts: self.counts.clone(),
                },
            );
        }
        tg.exec().await;
    }
}

#[lamellar::AmLocalData(Clone, Debug)]
struct LaunchAmUnsafeU32Group {
    rand_indices: Arc<Vec<usize>>,
    slice_start: usize,
    slice_end: usize,
    counts: SharedMemoryRegion<usize>,
}

#[lamellar::local_am]
impl LamellarAM for LaunchAmUnsafeU32Group {
    async fn exec(self) {
        let mut tg = typed_am_group!(UnsafeU32Group, lamellar::world.clone());
        for idx in &self.rand_indices[self.slice_start..self.slice_end] {
            let rank = idx % lamellar::num_pes;
            let index = idx / lamellar::num_pes;
            tg.add_am_pe(
                rank,
                UnsafeU32Group {
                    index: index as u32,
                    counts: self.counts.clone(),
                },
            );
        }
        tg.exec().await;
    }
}

#[lamellar::AmLocalData(Clone, Debug)]
struct LaunchAmUnsafeUsizeGroup {
    rand_indices: Arc<Vec<usize>>,
    slice_start: usize,
    slice_end: usize,
    counts: SharedMemoryRegion<usize>,
}

#[lamellar::local_am]
impl LamellarAM for LaunchAmUnsafeUsizeGroup {
    async fn exec(self) {
        let mut tg = typed_am_group!(UnsafeUsizeGroup, lamellar::world.clone());
        for idx in &self.rand_indices[self.slice_start..self.slice_end] {
            let rank = idx % lamellar::num_pes;
            let index = idx / lamellar::num_pes;
            tg.add_am_pe(
                rank,
                UnsafeUsizeGroup {
                    index,
                    counts: self.counts.clone(),
                },
            );
        }
        tg.exec().await;
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
            AmType::SafeU32Group(ref counts) => world
                .exec_am_local(LaunchAmSafeU32Group {
                    rand_indices: rand_indices.clone(),
                    slice_start: start,
                    slice_end: end,
                    counts: counts.clone(),
                })
                .spawn(),
            AmType::SafeUsizeGroup(ref counts) => world
                .exec_am_local(LaunchAmSafeUsizeGroup {
                    rand_indices: rand_indices.clone(),
                    slice_start: start,
                    slice_end: end,
                    counts: counts.clone(),
                })
                .spawn(),
            AmType::UnsafeU32Group(ref counts) => world
                .exec_am_local(LaunchAmUnsafeU32Group {
                    rand_indices: rand_indices.clone(),
                    slice_start: start,
                    slice_end: end,
                    counts: counts.clone(),
                })
                .spawn(),
            AmType::UnsafeUsizeGroup(ref counts) => world
                .exec_am_local(LaunchAmUnsafeUsizeGroup {
                    rand_indices: rand_indices.clone(),
                    slice_start: start,
                    slice_end: end,
                    counts: counts.clone(),
                })
                .spawn(),
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
    std::env::set_var(
        "LAMELLAR_BATCH_OP_SIZE",
        format!("{}", histo_config.buffer_size),
    );
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
            IndexSize::U32 => launch_ams(
                world,
                histo_config,
                rand_indices,
                AmType::SafeU32Group(counts),
            ),
            IndexSize::Usize => launch_ams(
                world,
                histo_config,
                rand_indices,
                AmType::SafeUsizeGroup(counts),
            ),
        };
        (init_time, launch_tasks)
    } else {
        let counts = world
            .alloc_shared_mem_region(histo_config.pe_table_size(num_pes))
            .block();
        unsafe {
            for elem in counts.as_mut_slice().unwrap().iter_mut() {
                *elem = 0;
            }
        }
        world.barrier();
        let init_time = timer.elapsed();
        timer = Instant::now();
        let launch_tasks = match index_size {
            IndexSize::U32 => launch_ams(
                world,
                histo_config,
                rand_indices,
                AmType::UnsafeU32Group(counts),
            ),
            IndexSize::Usize => launch_ams(
                world,
                histo_config,
                rand_indices,
                AmType::UnsafeUsizeGroup(counts),
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
