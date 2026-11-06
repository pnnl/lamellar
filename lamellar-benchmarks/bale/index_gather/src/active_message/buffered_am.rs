// clippy misattributes let_and_return warnings to macro-expanded #[lamellar::am] bodies
#![allow(clippy::let_and_return)]

use lamellar::active_messaging::prelude::*;
use lamellar::active_messaging::{AmDist, LamellarAM, RemoteActiveMessage, Serde};
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
#[lamellar::AmData(Clone, Debug)]
struct SafeBufferedAMu32 {
    indices: Vec<u32>,
    table: Darc<Vec<usize>>,
}

#[lamellar::am]
impl LamellarAM for SafeBufferedAMu32 {
    async fn exec(self) -> Vec<usize> {
        self.indices
            .iter()
            .map(|i| self.table[*i as usize])
            .collect::<Vec<usize>>()
    }
}

// Updates are not atomic, indices are buffered usizes
#[lamellar::AmData(Clone, Debug)]
struct SafeBufferedAMusize {
    indices: Vec<usize>,
    table: Darc<Vec<usize>>,
}

#[lamellar::am]
impl LamellarAM for SafeBufferedAMusize {
    async fn exec(self) -> Vec<usize> {
        self.indices
            .iter()
            .map(|i| self.table[*i])
            .collect::<Vec<usize>>()
    }
}

//-----------------------------------------------------------

//------ Unsafe AMs ------
// Updates are not atomic, indices are buffered u32s
#[lamellar::AmData(Clone, Debug)]
struct UnsafeBufferedAMu32 {
    indices: Vec<u32>,
    table: SharedMemoryRegion<usize>,
}

#[lamellar::am]
impl LamellarAM for UnsafeBufferedAMu32 {
    async fn exec(self) -> Vec<usize> {
        self.indices
            .iter()
            .map(|i| unsafe { self.table.as_mut_slice()[*i as usize] })
            .collect::<Vec<usize>>()
    }
}

// Updates are not atomic, indices are buffered usizes
#[lamellar::AmData(Clone, Debug)]
struct UnsafeBufferedAMusize {
    indices: Vec<usize>,
    table: SharedMemoryRegion<usize>,
}

#[lamellar::am]
impl LamellarAM for UnsafeBufferedAMusize {
    async fn exec(self) -> Vec<usize> {
        self.indices
            .iter()
            .map(|i| unsafe { self.table.as_mut_slice()[*i] })
            .collect::<Vec<usize>>()
    }
}
//------------------------

// Helper trait to reduce code duplication of the LaunchAm
trait BufferedAm: RemoteActiveMessage + LamellarAM + Serde + AmDist + Clone {
    // type Index: Sync + Send + Clone;
    type AM: LamellarAM;
    fn blank(&self) -> Self;
    // fn to_am(self) -> Self::AM;
    fn add_index(&mut self, index: usize);
    fn len(&self) -> usize;
}

impl BufferedAm for SafeBufferedAMu32 {
    // type Index = u32;
    type AM = Self;
    fn blank(&self) -> Self {
        Self {
            indices: Vec::new(),
            table: self.table.clone(),
        }
    }
    // fn to_am(self) -> Self::AM {
    //     self
    // }
    fn add_index(&mut self, index: usize) {
        self.indices.push(index as u32);
    }
    fn len(&self) -> usize {
        self.indices.len()
    }
}

impl BufferedAm for SafeBufferedAMusize {
    type AM = Self;
    fn blank(&self) -> Self {
        Self {
            indices: Vec::new(),
            table: self.table.clone(),
        }
    }
    // fn to_am(self) -> Self::AM {
    //     self
    // }
    fn add_index(&mut self, index: usize) {
        self.indices.push(index);
    }
    fn len(&self) -> usize {
        self.indices.len()
    }
}

impl BufferedAm for UnsafeBufferedAMu32 {
    type AM = Self;
    fn blank(&self) -> Self {
        Self {
            indices: Vec::new(),
            table: self.table.clone(),
        }
    }
    // fn to_am(self) -> Self::AM {
    //     self
    // }
    fn add_index(&mut self, index: usize) {
        self.indices.push(index as u32);
    }
    fn len(&self) -> usize {
        self.indices.len()
    }
}

impl BufferedAm for UnsafeBufferedAMusize {
    type AM = Self;
    fn blank(&self) -> Self {
        Self {
            indices: Vec::new(),
            table: self.table.clone(),
        }
    }
    // fn to_am(self) -> Self::AM {
    //     self
    // }
    fn add_index(&mut self, index: usize) {
        self.indices.push(index);
    }
    fn len(&self) -> usize {
        self.indices.len()
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
    async fn exec(self) -> Vec<<T as LamellarAM>::Output> {
        // let _timer = Instant::now();
        let num_pes = lamellar::num_pes;
        let mut pe_ams = vec![self.am_builder.blank(); num_pes];
        let task_group = LamellarTaskGroup::new(lamellar::team.clone());
        let mut reqs = vec![];
        for idx in self.rand_indices[self.slice_start..self.slice_end].iter() {
            let rank = idx % num_pes;
            let offset = idx / num_pes;
            pe_ams[rank].add_index(offset);
            if pe_ams[rank].len() >= self.buffer_size {
                let mut am = self.am_builder.blank();
                std::mem::swap(&mut am, &mut pe_ams[rank]);
                reqs.push(task_group.exec_am_pe(rank, am).spawn());
            }
        }
        //send any remaining buffered updates

        for (rank, am) in pe_ams.into_iter().enumerate() {
            if am.len() > 0 {
                reqs.push(task_group.exec_am_pe(rank, am).spawn());
            }
        }
        // println!("send time {:?}", _timer.elapsed());
        futures::future::join_all(reqs).await
        // .into_iter()
        // .flatten()
        // .collect::<Vec<usize>>()
    }
}

type LaunchAmsFut<T> = Pin<Box<dyn Future<Output = Vec<Vec<<T as LamellarAM>::Output>>>>>;

fn launch_ams<T: BufferedAm>(
    world: &LamellarWorld,
    ig_config: &IndexGatherCli,
    rand_indices: &Arc<Vec<usize>>,
    am_builder: T,
) -> LaunchAmsFut<T> {
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
                    buffer_size: ig_config.buffer_size,
                    am_builder: am_builder.clone(),
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
            IndexSize::U32 => launch_ams(
                world,
                ig_config,
                rand_indices,
                SafeBufferedAMu32 {
                    indices: Vec::new(),
                    table: table.clone(),
                },
            ),
            IndexSize::Usize => launch_ams(
                world,
                ig_config,
                rand_indices,
                SafeBufferedAMusize {
                    indices: Vec::new(),
                    table: table.clone(),
                },
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
                UnsafeBufferedAMu32 {
                    indices: Vec::new(),
                    table: table.clone(),
                },
            ),
            IndexSize::Usize => launch_ams(
                world,
                ig_config,
                rand_indices,
                UnsafeBufferedAMusize {
                    indices: Vec::new(),
                    table: table.clone(),
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
