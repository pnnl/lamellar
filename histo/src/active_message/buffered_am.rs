#![allow(clippy::let_underscore_future)]

use lamellar::active_messaging::prelude::*;
use lamellar::active_messaging::{AmDist, LamellarAM, RemoteActiveMessage, Serde};
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
struct SafeBufferedAMu32 {
    indices: std::vec::Vec<u32>,
    counts: Darc<Vec<AtomicUsize>>,
}

#[lamellar::am]
impl LamellarAM for SafeBufferedAMu32 {
    async fn exec(self) {
        for o in &self.indices {
            self.counts[*o as usize].fetch_add(1, Ordering::Relaxed);
        }
    }
}

// Updates are not atomic, indices are buffered usizes
#[lamellar::AmData(Clone, Debug)]
struct SafeBufferedAMusize {
    indices: std::vec::Vec<usize>,
    counts: Darc<Vec<AtomicUsize>>,
}

#[lamellar::am]
impl LamellarAM for SafeBufferedAMusize {
    async fn exec(self) {
        for o in &self.indices {
            self.counts[*o].fetch_add(1, Ordering::Relaxed);
        }
    }
}

//-----------------------------------------------------------

//------ Unsafe AMs ------
// Updates are not atomic, indices are buffered u32s
#[lamellar::AmData(Clone, Debug)]
struct UnsafeBufferedAMu32 {
    indices: std::vec::Vec<u32>,
    counts: SharedMemoryRegion<usize>,
}

#[lamellar::am]
impl LamellarAM for UnsafeBufferedAMu32 {
    async fn exec(self) {
        for o in &self.indices {
            unsafe { self.counts.as_mut_slice().unwrap()[*o as usize] += 1 }; //this update would be unsafe and has potential for races / dropped updates
        }
    }
}

// Updates are not atomic, indices are buffered usizes
#[lamellar::AmData(Clone, Debug)]
struct UnsafeBufferedAMusize {
    indices: std::vec::Vec<usize>,
    counts: SharedMemoryRegion<usize>,
}

#[lamellar::am]
impl LamellarAM for UnsafeBufferedAMusize {
    async fn exec(self) {
        for o in &self.indices {
            unsafe { self.counts.as_mut_slice().unwrap()[*o] += 1 }; //this update would be unsafe and has potential for races / dropped updates
        }
    }
}
//------------------------

// Helper trait to reduce code duplication of the LaunchAm
trait BufferedAm: RemoteActiveMessage + LamellarAM + Serde + AmDist + Clone {
    // type Index: Sync + Send + Clone;
    type AM: LamellarAM;
    fn blank(&self) -> Self;
    fn add_index(&mut self, index: usize);
    fn len(&self) -> usize;
    fn index_size(&self) -> usize;
}

impl BufferedAm for SafeBufferedAMu32 {
    // type Index = u32;
    type AM = Self;
    fn blank(&self) -> Self {
        Self {
            indices: Vec::new(),
            counts: self.counts.clone(),
        }
    }
    fn add_index(&mut self, index: usize) {
        self.indices.push(index as u32);
    }
    fn len(&self) -> usize {
        self.indices.len()
    }
    fn index_size(&self) -> usize {
        std::mem::size_of::<u32>()
    }
}

impl BufferedAm for SafeBufferedAMusize {
    type AM = Self;
    fn blank(&self) -> Self {
        Self {
            indices: Vec::new(),
            counts: self.counts.clone(),
        }
    }
    fn add_index(&mut self, index: usize) {
        self.indices.push(index);
    }
    fn len(&self) -> usize {
        self.indices.len()
    }
    fn index_size(&self) -> usize {
        std::mem::size_of::<usize>()
    }
}

impl BufferedAm for UnsafeBufferedAMu32 {
    type AM = Self;
    fn blank(&self) -> Self {
        Self {
            indices: Vec::new(),
            counts: self.counts.clone(),
        }
    }
    fn add_index(&mut self, index: usize) {
        self.indices.push(index as u32);
    }
    fn len(&self) -> usize {
        self.indices.len()
    }
    fn index_size(&self) -> usize {
        std::mem::size_of::<u32>()
    }
}

impl BufferedAm for UnsafeBufferedAMusize {
    type AM = Self;
    fn blank(&self) -> Self {
        Self {
            indices: Vec::new(),
            counts: self.counts.clone(),
        }
    }
    fn add_index(&mut self, index: usize) {
        self.indices.push(index);
    }
    fn len(&self) -> usize {
        self.indices.len()
    }

    fn index_size(&self) -> usize {
        std::mem::size_of::<usize>()
    }
}

// We likely want to issue updates from multiple threads to improve performance
// we can use a local Active Messages to do this.

#[lamellar::AmLocalData(Clone, Debug)]
struct LaunchAm<T: BufferedAm> {
    rand_indices: Arc<Vec<usize>>,
    slice_start: usize,
    slice_end: usize,
    buffer_size: usize,
    am_builder: T,
}

#[lamellar::local_am]
impl<T: BufferedAm> LamellarAM for LaunchAm<T> {
    async fn exec(self) {
        let num_pes = lamellar::num_pes;
        let mut pe_ams = vec![self.am_builder.blank(); num_pes];
        let task_group = LamellarTaskGroup::new(lamellar::team.clone());
        let mut pe_cnt = vec![0; num_pes];
        for idx in self.rand_indices[self.slice_start..self.slice_end].iter() {
            let rank = idx % num_pes;
            let offset = idx / num_pes;
            pe_ams[rank].add_index(offset);
            if pe_ams[rank].len() * self.am_builder.index_size() >= self.buffer_size {
                let mut am = self.am_builder.blank();
                std::mem::swap(&mut am, &mut pe_ams[rank]);
                let _ = task_group.exec_am_pe(rank, am); //we could await here but we will just do a wait_all later instead
                pe_cnt[rank] += 1;
            }
        }
        //send any remaining buffered updates
        let _timer = Instant::now();
        for (rank, am) in pe_ams.into_iter().enumerate() {
            if am.len() > 0 {
                let _ = task_group.exec_am_pe(rank, am); //we could await here but we will just do a wait_all later instead
                pe_cnt[rank] += 1;
            }
        }
        task_group.wait_all();
    }
}

fn launch_ams<T: BufferedAm>(
    world: &LamellarWorld,
    histo_config: &HistoCli,
    rand_indices: &Arc<Vec<usize>>,
    am_builder: T,
) -> Pin<Box<dyn Future<Output = Vec<()>>>> {
    let num_pes = world.num_pes();
    let slice_size = histo_config.pe_updates(num_pes) as f32 / histo_config.launch_threads as f32;
    let mut launch_tasks = vec![];

    for tid in 0..histo_config.launch_threads {
        let start = (tid as f32 * slice_size).round() as usize;
        let end = (tid as f32 * slice_size + slice_size).round() as usize;
        launch_tasks.push(
            world
                .exec_am_local(LaunchAm {
                    rand_indices: rand_indices.clone(),
                    slice_start: start,
                    slice_end: end,
                    buffer_size: histo_config.buffer_size,
                    am_builder: am_builder.clone(),
                }),
        );
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
        let counts = Darc::new(world, counts_inner).expect("darc should be created");
        world.barrier();
        let init_time = timer.elapsed();
        timer = Instant::now();
        let launch_tasks = match index_size {
            IndexSize::U32 => launch_ams(
                world,
                histo_config,
                rand_indices,
                SafeBufferedAMu32 {
                    indices: Vec::new(),
                    counts: counts.clone(),
                },
            ),
            IndexSize::Usize => launch_ams(
                world,
                histo_config,
                rand_indices,
                SafeBufferedAMusize {
                    indices: Vec::new(),
                    counts: counts.clone(),
                },
            ),
        };
        (init_time, launch_tasks)
    } else {
        let counts = world.alloc_shared_mem_region(histo_config.pe_table_size(num_pes));
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
                UnsafeBufferedAMu32 {
                    indices: Vec::new(),
                    counts: counts.clone(),
                },
            ),
            IndexSize::Usize => launch_ams(
                world,
                histo_config,
                rand_indices,
                UnsafeBufferedAMusize {
                    indices: Vec::new(),
                    counts: counts.clone(),
                },
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
