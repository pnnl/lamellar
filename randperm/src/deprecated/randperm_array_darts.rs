mod options;
use clap::Parser;

use lamellar::array::prelude::*;
use rand::prelude::*;
use std::time::Instant;

fn main() {
    let world = lamellar::LamellarWorldBuilder::new().build();
    let my_pe = world.my_pe();
    let num_pes = world.num_pes();
    let cli = options::RandpermCli::parse();

    let global_count = cli.global_size;
    let iterations = cli.iterations;
    let target_factor = cli.target_factor;

    if my_pe == 0 {
        cli.describe(num_pes);
    }

    // start with unsafe because they are faster to initialize than AtomicArrays
    let darts_array = UnsafeArray::<usize>::new(
        world.team(),
        global_count,
        lamellar::array::Distribution::Block,
    );
    let target_array = UnsafeArray::<usize>::new(
        world.team(),
        global_count * target_factor,
        lamellar::array::Distribution::Block,
    );

    let mut rng: StdRng = SeedableRng::seed_from_u64(my_pe as u64);

    // initialize arrays
    let darts_init = unsafe {
        darts_array
            .dist_iter_mut()
            .enumerate()
            .for_each(|(i, x)| *x = i)
    }; // each PE some slice in [0..global_count]
    let target_init = unsafe { target_array.dist_iter_mut().for_each(|x| *x = usize::MAX) };
    world.block_on(darts_init);
    world.block_on(target_init);
    world.wait_all();

    //will use this slice for first iteration
    let darts_array = darts_array.into_read_only();
    let local_darts = darts_array.local_data();

    let target_array = target_array.into_atomic();
    world.barrier();
    if my_pe == 0 {
        println!("start");
    }

    for _ in 0..iterations {
        world.barrier();
        let now = Instant::now();
        // ====== perform the actual random permute========//
        let rand_index = (0..local_darts.len())
            .map(|_| rng.gen_range(0, global_count * target_factor))
            .collect::<Vec<usize>>();

        // launch initial set of darts, and collect any that didnt stick
        let mut remaining_darts = world
            .block_on(target_array.batch_compare_exchange(&rand_index, usize::MAX, local_darts))
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
        while remaining_darts.len() > 0 {
            // println!("{}", remaining_darts.len());
            let rand_index = (0..remaining_darts.len())
                .map(|_| rng.gen_range(0, global_count * target_factor))
                .collect::<Vec<usize>>();
            remaining_darts = world
                .block_on(target_array.batch_compare_exchange(
                    &rand_index,
                    usize::MAX,
                    remaining_darts.clone(),
                ))
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
        world.wait_all(); //my work is done
        if my_pe == 0 {
            println!("local run time {:?} ", now.elapsed(),);
        }
        world.barrier(); //all work is done
        if my_pe == 0 {
            println!("permute time {:?}s ", now.elapsed().as_secs_f64(),);
        }
        let collect_start = Instant::now();
        let the_array = world.block_on(
            target_array
                .dist_iter()
                .filter_map(|elem| {
                    let elem = elem.load(); //elements are atomic so we cant just read directly
                    if elem < usize::MAX {
                        Some(elem)
                    } else {
                        None
                    }
                })
                .collect::<ReadOnlyArray<usize>>(lamellar::array::Distribution::Block),
        );
        world.barrier();
        let global_time = now.elapsed().as_secs_f64();
        if my_pe == 0 {
            println!("collect time: {:?}s", collect_start.elapsed().as_secs_f64());
            println!(
                "global time {:?} MB {:?} MB/s: {:?} ",
                global_time,
                (world.MB_sent()),
                (world.MB_sent()) / global_time,
            );
            println!("Secs: {:?}", global_time,);
            let sum = world.block_on(the_array.sum());
            println!(
                "reduced sum: {sum} calculated sum {} ",
                (global_count * (global_count + 1) / 2) - global_count
            );
            if sum != (global_count * (global_count + 1) / 2) - global_count {
                println!("Error! randperm not as expected");
            }
        }
        world.block_on(
            target_array
                .dist_iter_mut()
                .for_each(|x| x.store(usize::MAX)),
        );
        world.barrier();
    }
}
