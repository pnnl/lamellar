use crate::options::IndexGatherCli;
use lamellar::array::prelude::*;

use clap::ValueEnum;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub enum ArrayType {
    Unsafe,
    Atomic,
    ReadOnly,
    LocalLock,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum ArrayDistribution {
    Block,
    Cyclic,
}
impl From<&ArrayDistribution> for lamellar::Distribution {
    fn from(val: &ArrayDistribution) -> Self {
        match val {
            ArrayDistribution::Block => lamellar::Distribution::Block,
            ArrayDistribution::Cyclic => lamellar::Distribution::Cyclic,
        }
    }
}
pub fn index_gather(
    world: &lamellar::LamellarWorld,
    ig_config: &IndexGatherCli,
    rand_indices: &Arc<Vec<usize>>,
    array_type: ArrayType,
    distribution: &ArrayDistribution,
) -> (Duration, Duration, Duration, Duration) {
    let num_pes = world.num_pes();
    std::env::set_var(
        "LAMELLAR_BATCH_OP_THREADS",
        format!("{}", ig_config.launch_threads),
    );
    std::env::set_var(
        "LAMELLAR_BATCH_OP_SIZE",
        format!("{}", ig_config.buffer_size),
    );
    world.barrier();
    let mut timer = Instant::now();

    let _index_gather_request = match array_type {
        ArrayType::Unsafe => {
            let array = UnsafeArray::<usize>::new(
                world,
                ig_config.total_table_size(num_pes),
                distribution.into(),
            )
            .block();
            let _init_time = timer.elapsed();
            timer = Instant::now();
            //the actual index_gather operation
            unsafe { array.batch_load(rand_indices.as_slice()).spawn() }
        }
        ArrayType::Atomic => {
            let array = AtomicArray::<usize>::new(
                world,
                ig_config.total_table_size(num_pes),
                distribution.into(),
            )
            .block();
            let _init_time = timer.elapsed();
            timer = Instant::now();

            //the actual index_gather operation
            array.batch_load(rand_indices.as_slice()).spawn()
        }
        ArrayType::ReadOnly => {
            let array = ReadOnlyArray::<usize>::new(
                world,
                ig_config.total_table_size(num_pes),
                distribution.into(),
            )
            .block();
            let _init_time = timer.elapsed();
            timer = Instant::now();

            //the actual index_gather operation
            array.batch_load(rand_indices.as_slice()).spawn()
        }
        ArrayType::LocalLock => {
            let array = LocalLockArray::<usize>::new(
                world,
                ig_config.total_table_size(num_pes),
                distribution.into(),
            )
            .block();
            let _init_time = timer.elapsed();
            timer = Instant::now();

            //the actual index_gather operation
            array.batch_load(rand_indices.as_slice()).spawn()
        }
    };

    let launch_issue_time = timer.elapsed();
    let launch_finish_time = timer.elapsed();
    // world.block_on(index_gather_request);
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
