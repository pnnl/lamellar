use rand::prelude::*; // prelude imports contain common types and traits for the given crate
use rayon::prelude::*;

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use lamellar::active_messaging::prelude::*;
use lamellar::array::prelude::*;
use lamellar::darc::prelude::*;

fn generate_random_indices(n: usize, max_val: usize) -> Vec<usize> {
    let mut rng = rand::rng();
    (0..n).map(|_| rng.random_range(0..max_val)).collect()
    // (0..n).map(|i| i).collect()
}

fn serial_histogram(indices: &[usize]) {
    let mut table = vec![0; indices.len()];
    let timer = std::time::Instant::now();
    for i in indices {
        table[*i] += 1;
    }
    println!("Serial Time: {:?}", timer.elapsed());
    println!("Sum: {:?}", table.iter().sum::<usize>());
}

fn serial_histogram_atomic(indices: &[usize]) {
    let mut table = Vec::with_capacity(indices.len());
    for _ in 0..indices.len() {
        table.push(AtomicUsize::new(0));
    }
    let timer = std::time::Instant::now();
    for i in indices {
        table[*i].fetch_add(1, Ordering::Relaxed);
    }
    println!("Serial (Atomic) Time: {:?}", timer.elapsed());
    println!(
        "Sum: {:?}",
        table
            .iter()
            .map(|e| e.load(Ordering::SeqCst))
            .sum::<usize>()
    );
}

fn rayon_histogram(indices: &[usize]) {
    let mut table = Vec::with_capacity(indices.len());
    for _ in 0..indices.len() {
        table.push(AtomicUsize::new(0));
    }
    let timer = std::time::Instant::now();
    indices.par_iter().for_each(|i| {
        table[*i].fetch_add(1, Ordering::Relaxed);
    });
    println!("Rayon Time: {:?}", timer.elapsed());
    println!(
        "Sum: {:?}",
        table
            .iter()
            .map(|e| e.load(Ordering::SeqCst))
            .sum::<usize>()
    );
}

fn lamellar_histogram(world: &LamellarWorld, indices: &[usize]) {
    let table: AtomicArray<usize> = AtomicArray::new(
        world,
        indices.len() * world.num_pes(),
        lamellar::Distribution::Cyclic,
    )
    .block();
    world.barrier();
    let timer = std::time::Instant::now();
    table.batch_add(indices, 1).block();
    table.barrier();
    println!("Lamellar Time: {:?}", timer.elapsed());

    if world.my_pe() == 0 {
        println!("Sum: {:?}", table.sum().block(),);
    }
}

fn lamellar_am_histogram_one_pe(world: &LamellarWorld, indices: Vec<usize>) -> Vec<usize> {
    let mut table = Vec::with_capacity(indices.len());
    for _ in 0..indices.len() {
        table.push(AtomicUsize::new(0));
    }
    let table = Arc::new(table);
    let indices = Arc::new(indices);

    // We know we are only running one one PE so we want to make use of all the threads
    let num_threads = std::cmp::max(world.num_threads_per_pe(), 1);
    let chunk_size = indices.len() / num_threads; // we have have remainder here but we'll ignore that for now
    world.barrier();
    let timer = std::time::Instant::now();

    for thread_id in 0..num_threads {
        let _ = world
            .exec_am_local(HistoLaunchOne {
                indices: indices.clone(),
                thread_id: thread_id,
                chunk_size: chunk_size,
                table: table.clone(),
            })
            .spawn();
    }
    world.wait_all();

    println!("Lamellar Time: {:?} ", timer.elapsed());
    if world.my_pe() == 0 {
        println!(
            "Sum: {:?}",
            table
                .iter()
                .map(|e| e.load(Ordering::SeqCst))
                .sum::<usize>()
        );
    }
    Arc::into_inner(indices).unwrap()
}

fn lamellar_am_histogram(world: &LamellarWorld, indices: Vec<usize>) {
    let mut table = Vec::with_capacity(indices.len());
    for _ in 0..indices.len() {
        table.push(AtomicUsize::new(0));
    }
    let table = Darc::new(world, table)
        .block()
        .expect("failed to create Darc");
    let indices = Arc::new(indices);

    // We will be receiving AM's from multiple PEs so we want don't necessarily
    // want to assign an initial AM to each thread (as we also want to be able to quickly process incoming AMs)
    let num_threads = std::cmp::max(world.num_threads_per_pe() / 4, 1);
    let chunk_size = indices.len() / num_threads; // we have have remainder here but we'll ignore that for now
    world.barrier();
    let timer = std::time::Instant::now();

    for thread_id in 0..num_threads {
        let _ = world
            .exec_am_local(HistoLaunch {
                indices: indices.clone(),
                thread_id: thread_id,
                chunk_size: chunk_size,
                table: table.clone(),
            })
            .spawn();
    }
    world.wait_all();
    world.barrier();
    println!("Lamellar Time: {:?} ", timer.elapsed());
    println!(
        "Sum: {:?}",
        table
            .iter()
            .map(|e| e.load(Ordering::SeqCst))
            .sum::<usize>()
    );
}

#[AmLocalData]
struct HistoLaunchOne {
    indices: Arc<Vec<usize>>,
    thread_id: usize,
    chunk_size: usize,
    table: Arc<Vec<AtomicUsize>>,
}

#[local_am]
impl LamellarAM for HistoLaunchOne {
    async fn exec(&self) {
        for i in
            &self.indices[self.thread_id * self.chunk_size..(self.thread_id + 1) * self.chunk_size]
        {
            self.table[*i].fetch_add(1, Ordering::Relaxed);
        }
    }
}

#[AmLocalData]
struct HistoLaunch {
    indices: Arc<Vec<usize>>,
    thread_id: usize,
    chunk_size: usize,
    table: Darc<Vec<AtomicUsize>>,
}

#[local_am]
impl LamellarAM for HistoLaunch {
    async fn exec(&self) {
        let mut pe_indices = vec![vec![]; lamellar::num_pes];
        for i in
            &self.indices[self.thread_id * self.chunk_size..(self.thread_id + 1) * self.chunk_size]
        {
            let pe = *i % lamellar::num_pes;
            let offset = *i / lamellar::num_pes;
            pe_indices[pe].push(offset);
        }
        for (pe, indices) in pe_indices.into_iter().enumerate() {
            let _ = lamellar::world
                .exec_am_pe(
                    pe,
                    HistoAm {
                        indices,
                        table: self.table.clone(),
                    },
                )
                .spawn();
        }
    }
}

#[AmData]
struct HistoAm {
    indices: Vec<usize>,
    #[AmGroup(static)]
    table: Darc<Vec<AtomicUsize>>,
}

#[am]
impl LamellarAM for HistoAm {
    async fn exec(&self) {
        for i in &self.indices {
            self.table[*i].fetch_add(1, Ordering::Relaxed);
        }
    }
}

fn main() {
    let table_size = 1000000000;

    let indices = generate_random_indices(table_size, table_size);
    serial_histogram(&indices);
    serial_histogram_atomic(&indices);
    rayon_histogram(&indices);

    // let world = LamellarWorldBuilder::new().build();
    // let mut indices = generate_random_indices(table_size / world.num_pes(), table_size);
    // if world.my_pe() == 0 && world.num_pes() == 1 {
    //     serial_histogram(&indices);
    //     serial_histogram_atomic(&indices);
    // }
    // lamellar_histogram(&world, &indices);
    // if world.num_pes() == 1 {
    //     indices = lamellar_am_histogram_one_pe(&world, indices);
    // }
    // lamellar_am_histogram(&world, indices);
}
