#![allow(clippy::let_underscore_future)]

pub mod buffered_cas_am;
pub mod buffered_cas_am_remote;
pub mod buffered_push_am;
pub mod cas_am_group;
pub mod cas_am_group_remote;
pub mod push_am_group;
pub mod single_cas_am;
pub mod single_cas_am_remote;
pub mod single_push_am;

use crate::options::RandPermCli;

use lamellar::active_messaging::prelude::*;
use lamellar::array::prelude::*;
use lamellar::darc::prelude::*;

use std::sync::atomic::{AtomicUsize, Ordering};

#[lamellar::AmData]
struct SumAm {
    sum: Darc<AtomicUsize>,
    amt: usize,
}

#[lamellar::am]
impl LamellarAM for SumAm {
    async fn exec(self) {
        self.sum.fetch_add(self.amt, Ordering::Relaxed);
    }
}

#[lamellar::AmData]
struct CollectAm {
    array: LocalRwDarc<Vec<usize>>,
    data: Vec<usize>,
    index: usize,
}

#[lamellar::am]
impl LamellarAM for CollectAm {
    async fn exec(self) {
        self.array.write().await[self.index..]
            .iter_mut()
            .zip(self.data.iter())
            .for_each(|(a, b)| *a = *b);
    }
}

fn collect_perm(
    world: &LamellarWorld,
    rand_perm_config: &RandPermCli,
    mut data: Vec<usize>,
    the_array: &LocalRwDarc<Vec<usize>>,
    local_lens: &AtomicArray<usize>,
) {
    local_lens.local_data().at(0).store(data.len());
    world.barrier();
    let start_index = local_lens
        .buffered_onesided_iter(world.num_pes())
        .into_iter()
        .take(world.my_pe())
        .sum::<usize>();
    let end_index = start_index + data.len() - 1; //inclusive
    let pe_size = rand_perm_config.pe_table_size(world.num_pes());

    let start_pe = (start_index / pe_size) as isize;
    let end_pe = (end_index / pe_size) as isize;
    let start_offset = start_index % pe_size;
    let end_offset = end_index % pe_size;

    if start_pe == end_pe {
        let _ = world
            .exec_am_pe(
                start_pe as usize,
                CollectAm {
                    array: the_array.clone(),
                    data,
                    index: start_offset,
                },
            )
            .spawn();
    } else {
        let mut cur_pe = end_pe;
        while cur_pe >= start_pe {
            if cur_pe == end_pe {
                let pe_data = data.split_off(data.len() - end_offset - 1);
                let _ = world
                    .exec_am_pe(
                        cur_pe as usize,
                        CollectAm {
                            array: the_array.clone(),
                            data: pe_data, //end_offset + 1 is the number of elements in the last pe
                            index: 0,
                        },
                    )
                    .spawn();
            } else if cur_pe == start_pe {
                let _ = world
                    .exec_am_pe(
                        cur_pe as usize,
                        CollectAm {
                            array: the_array.clone(),
                            data, //this is the remaining data
                            index: start_offset,
                        },
                    )
                    .spawn();
                data = vec![]; //to appease the compiler because we consume data above
            } else {
                let pe_data = data.split_off(data.len() - pe_size);
                let _ = world
                    .exec_am_pe(
                        cur_pe as usize,
                        CollectAm {
                            array: the_array.clone(),
                            data: pe_data, //we take the entire pes range
                            index: 0,
                        },
                    )
                    .spawn();
            }
            cur_pe -= 1;
        }
    }
    world.wait_all();
    world.barrier();
}
