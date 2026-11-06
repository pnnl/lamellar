use crate::options::HistoCli;
use lamellar::array::prelude::*;

use clap::ValueEnum;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub enum ArrayType {
    Unsafe,
    Atomic,
    LocalLock,
}

#[lamellar::AmData(Default, Debug, ArrayOps(Arithmetic), PartialEq, PartialOrd)]
pub struct MyUsize {
    v: usize,
}

impl std::ops::AddAssign for MyUsize {
    fn add_assign(&mut self, other: MyUsize) {
        self.v = self.v.wrapping_add(other.v);
    }
}
impl std::ops::SubAssign for MyUsize {
    fn sub_assign(&mut self, other: MyUsize) {
        self.v = self.v.wrapping_sub(other.v);
    }
}
impl std::ops::MulAssign for MyUsize {
    fn mul_assign(&mut self, other: MyUsize) {
        self.v = self.v.wrapping_mul(other.v);
    }
}
impl std::ops::DivAssign for MyUsize {
    fn div_assign(&mut self, other: MyUsize) {
        self.v = self.v.wrapping_div(other.v.max(1));
    }
}
impl std::ops::RemAssign for MyUsize {
    fn rem_assign(&mut self, other: MyUsize) {
        self.v = self.v.wrapping_rem(other.v.max(1));
    }
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
pub fn histo(
    world: &lamellar::LamellarWorld,
    histo_config: &HistoCli,
    rand_indices: &Arc<Vec<usize>>,
    array_type: ArrayType,
    distribution: &ArrayDistribution,
) -> (Duration, Duration, Duration, Duration) {
    let num_pes = world.num_pes();
    std::env::set_var(
        "LAMELLAR_BATCH_OP_THREADS",
        format!("{}", histo_config.launch_threads),
    );
    std::env::set_var(
        "LAMELLAR_BATCH_OP_SIZE",
        format!("{}", histo_config.buffer_size),
    );
    world.barrier();
    let mut timer = Instant::now();

    //TODO specify that we want usize arrays
    let histo_request = match array_type {
        ArrayType::Unsafe => {
            let array: UnsafeArray<usize> = UnsafeArray::new(
                world,
                histo_config.total_table_size(num_pes),
                distribution.into(),
            )
            .block();
            let _init_time = timer.elapsed();
            timer = Instant::now();
            //the actual histo operation
            unsafe { array.batch_add(rand_indices.as_slice(), 1) }
        }
        ArrayType::Atomic => {
            let array: AtomicArray<usize> = AtomicArray::new(
                world,
                histo_config.total_table_size(num_pes),
                distribution.into(),
            )
            .block();
            let _init_time = timer.elapsed();
            timer = Instant::now();

            //the actual histo operation
            array.batch_add(rand_indices.as_slice(), 1)
        }
        ArrayType::LocalLock => {
            let array: LocalLockArray<usize> = LocalLockArray::new(
                world,
                histo_config.total_table_size(num_pes),
                distribution.into(),
            )
            .block();
            let _init_time = timer.elapsed();
            timer = Instant::now();

            //the actual histo operation
            array.batch_add(rand_indices.as_slice(), 1)
        }
    };

    let launch_issue_time = timer.elapsed();
    let launch_finish_time = timer.elapsed();
    histo_request.block();
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

pub fn histo_custom_type(
    world: &lamellar::LamellarWorld,
    histo_config: &HistoCli,
    rand_indices: &Arc<Vec<usize>>,
    array_type: ArrayType,
    distribution: &ArrayDistribution,
) -> (Duration, Duration, Duration, Duration) {
    let num_pes = world.num_pes();
    std::env::set_var(
        "LAMELLAR_BATCH_OP_THREADS",
        format!("{}", histo_config.launch_threads),
    );
    std::env::set_var(
        "LAMELLAR_BATCH_OP_SIZE",
        format!("{}", histo_config.buffer_size),
    );
    world.barrier();
    let mut timer = Instant::now();

    let histo_request = match array_type {
        ArrayType::Unsafe => {
            let array: UnsafeArray<MyUsize> = UnsafeArray::new(
                world,
                histo_config.total_table_size(num_pes),
                distribution.into(),
            )
            .block();
            let _init_time = timer.elapsed();
            timer = Instant::now();
            //the actual histo operation
            unsafe { array.batch_add(rand_indices.as_slice(), MyUsize { v: 1 }) }
        }
        ArrayType::Atomic => {
            let array: AtomicArray<MyUsize> = AtomicArray::new(
                world,
                histo_config.total_table_size(num_pes),
                distribution.into(),
            )
            .block();
            let _init_time = timer.elapsed();
            timer = Instant::now();

            //the actual histo operation
            array.batch_add(rand_indices.as_slice(), MyUsize { v: 1 })
        }
        ArrayType::LocalLock => {
            let array: LocalLockArray<MyUsize> = LocalLockArray::new(
                world,
                histo_config.total_table_size(num_pes),
                distribution.into(),
            )
            .block();
            let _init_time = timer.elapsed();
            timer = Instant::now();

            //the actual histo operation
            array.batch_add(rand_indices.as_slice(), MyUsize { v: 1 })
        }
    };

    let launch_issue_time = timer.elapsed();
    let launch_finish_time = timer.elapsed();
    histo_request.block();
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
