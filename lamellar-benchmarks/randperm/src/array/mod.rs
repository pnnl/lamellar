use crate::options::RandPermCli;
use lamellar::array::prelude::*;

use clap::ValueEnum;
use rand::prelude::*;
use std::time::{Duration, Instant};

pub enum ArrayType {
    Unsafe,
    Atomic,
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

fn array_rand_perm<A: LamellarArray<usize> + CompareExchangeOps<usize>>(
    world: &lamellar::LamellarWorld,
    local_darts: &[usize],
    target_array: &A,
    rng: &mut StdRng,
) {
    // ====== perform the actual random permute========//
    let rand_index = (0..local_darts.len())
        .map(|_| rng.gen_range(0, target_array.len()))
        .collect::<Vec<usize>>();

    // launch initial set of darts, and collect any that didnt stick
    let init_darts = target_array.batch_compare_exchange(&rand_index, usize::MAX, local_darts);
    let mut remaining_darts = init_darts
        .block()
        .iter()
        .enumerate()
        .filter_map(|(i, elem)| {
            match elem {
                Ok(_val) => None,               //the dart stuck!
                Err(_) => Some(local_darts[i]), //something else was there, try again
            }
        })
        .collect::<Vec<usize>>();
    // continue launching remaining darts until they all stick
    while !remaining_darts.is_empty() {
        let rand_index = (0..remaining_darts.len())
            .map(|_| rng.gen_range(0, target_array.len()))
            .collect::<Vec<usize>>();
        remaining_darts = target_array
            .batch_compare_exchange(&rand_index, usize::MAX, remaining_darts.clone())
            .block()
            .iter()
            .enumerate()
            .filter_map(|(i, elem)| {
                match elem {
                    Ok(_val) => None,                   //the dart stuck!
                    Err(_) => Some(remaining_darts[i]), //something else was there, try again
                }
            })
            .collect::<Vec<usize>>();
    }
    world.wait_all();
    world.barrier();
}

fn unsafe_array_rand_perm<A: LamellarArray<usize> + UnsafeCompareExchangeOps<usize>>(
    world: &lamellar::LamellarWorld,
    local_darts: &[usize],
    target_array: &A,
    rng: &mut StdRng,
) {
    // ====== perform the actual random permute========//
    let rand_index = (0..local_darts.len())
        .map(|_| rng.gen_range(0, target_array.len()))
        .collect::<Vec<usize>>();

    // launch initial set of darts, and collect any that didnt stick
    let init_darts =
        unsafe { target_array.batch_compare_exchange(&rand_index, usize::MAX, local_darts) };
    let mut remaining_darts = init_darts
        .block()
        .iter()
        .enumerate()
        .filter_map(|(i, elem)| {
            match elem {
                Ok(_val) => None,               //the dart stuck!
                Err(_) => Some(local_darts[i]), //something else was there, try again
            }
        })
        .collect::<Vec<usize>>();
    // continue launching remaining darts until they all stick
    while !remaining_darts.is_empty() {
        let rand_index = (0..remaining_darts.len())
            .map(|_| rng.gen_range(0, target_array.len()))
            .collect::<Vec<usize>>();
        remaining_darts = unsafe {
            target_array
                .batch_compare_exchange(&rand_index, usize::MAX, remaining_darts.clone())
                .block()
                .iter()
                .enumerate()
                .filter_map(|(i, elem)| {
                    match elem {
                        Ok(_val) => None,                   //the dart stuck!
                        Err(_) => Some(remaining_darts[i]), //something else was there, try again
                    }
                })
                .collect::<Vec<usize>>()
        };
    }
    world.wait_all();
    world.barrier();
}
pub fn rand_perm(
    world: &lamellar::LamellarWorld,
    rand_perm_config: &RandPermCli,
    array_type: ArrayType,
    distribution: &ArrayDistribution,
) -> (Duration, Duration, Duration, usize) {
    let num_pes = world.num_pes();
    let my_pe = world.my_pe();
    std::env::set_var(
        "LAMELLAR_BATCH_OP_THREADS",
        format!("{}", rand_perm_config.launch_threads),
    );
    std::env::set_var(
        "LAMELLAR_BATCH_OP_SIZE",
        format!("{}", rand_perm_config.buffer_size),
    );
    world.barrier();
    let timer; //= Instant::now();

    let darts_array = UnsafeArray::<usize>::new(
        world,
        rand_perm_config.total_table_size(num_pes),
        lamellar::Distribution::Block,
    )
    .block();
    let target_array = UnsafeArray::<usize>::new(
        world,
        rand_perm_config.total_table_size(num_pes) * rand_perm_config.target_factor,
        distribution.into(),
    )
    .block();
    let mut rng: StdRng = SeedableRng::seed_from_u64(my_pe as u64);
    let darts_init = unsafe {
        darts_array.dist_iter_mut().enumerate().for_each(|(i, x)| {
            *x = i;
        })
    };
    let target_init = unsafe { target_array.dist_iter_mut().for_each(|x| *x = usize::MAX) };
    darts_init.block();
    target_init.block();

    let mut darts_array = darts_array.into_read_only().block();
    let local_darts = darts_array.local_data();
    let (perm_time, collect_time) = match array_type {
        ArrayType::Unsafe => {
            timer = Instant::now();
            unsafe_array_rand_perm(world, local_darts, &target_array, &mut rng);
            let perm_time = timer.elapsed();
            darts_array = unsafe {
                target_array
                    .dist_iter()
                    .filter_map(|x| if *x == usize::MAX { None } else { Some(*x) })
                    .collect::<ReadOnlyArray<usize>>(distribution.into())
                    .block()
            };
            (perm_time, timer.elapsed())
        }
        ArrayType::Atomic => {
            let temp = target_array.into_atomic().block();
            timer = Instant::now();
            array_rand_perm(world, local_darts, &temp, &mut rng);
            let perm_time = timer.elapsed();
            darts_array = temp
                .dist_iter()
                .filter_map(|x| {
                    let x = x.load();
                    if x == usize::MAX {
                        None
                    } else {
                        Some(x)
                    }
                })
                .collect::<ReadOnlyArray<usize>>(distribution.into())
                .block();

            (perm_time, timer.elapsed())
        }
        ArrayType::LocalLock => {
            let temp = target_array.into_local_lock().block();
            timer = Instant::now();
            array_rand_perm(world, local_darts, &temp, &mut rng);
            let perm_time = timer.elapsed();
            darts_array = temp
                .dist_iter()
                .filter_map(|x| if *x == usize::MAX { None } else { Some(*x) })
                .collect::<ReadOnlyArray<usize>>(distribution.into())
                .block();
            (perm_time, timer.elapsed())
        }
    };
    world.barrier();
    let global_finish_time = timer.elapsed();

    // if my_pe == 0 {
    let sum = darts_array.sum().block().unwrap();

    // (perm_time, collect_time, global_finish_time, sum.unwrap())
    (perm_time, collect_time, global_finish_time, sum)
}
