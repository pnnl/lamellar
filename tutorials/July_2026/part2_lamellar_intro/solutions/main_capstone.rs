use rand::Rng; // just the trait - `rand::prelude::*` also exports a `Distribution` trait
               // that collides with lamellar::array::Distribution

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use lamellar::active_messaging::prelude::*;
use lamellar::array::prelude::*;
use lamellar::darc::prelude::*;

fn generate_random_indices(n: usize, max_val: usize) -> Vec<usize> {
    let mut rng = rand::rng();
    (0..n).map(|_| Rng::random_range(&mut rng, 0..max_val)).collect()
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

// Fixed: batch_add()/sum() are `unsafe fn` on UnsafeArray - wrap both in `unsafe { ... }`.
fn lamellar_unsafe_histogram(world: &LamellarWorld, indices: &[usize]) {
    let table: UnsafeArray<usize> = UnsafeArray::new(
        world,
        indices.len() * world.num_pes(),
        Distribution::Cyclic,
    )
    .block();
    world.barrier();
    let timer = std::time::Instant::now();
    unsafe {
        table.batch_add(indices, 1).block();
    }
    table.barrier();
    println!("Lamellar Unsafe Time: {:?}", timer.elapsed());

    if world.my_pe() == 0 {
        println!("Sum: {:?}", unsafe { table.sum().block() });
    }
}

fn lamellar_histogram(world: &LamellarWorld, indices: &[usize]) {
    let table: AtomicArray<usize> = AtomicArray::new(
        world,
        indices.len() * world.num_pes(),
        Distribution::Cyclic,
    )
    .block();
    world.barrier();
    let timer = std::time::Instant::now();
    table.batch_add(indices, 1).block();
    table.barrier();
    println!("Lamellar Time: {:?}", timer.elapsed());

    if world.my_pe() == 0 {
        println!("Sum: {:?}", table.sum().block());
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
    async fn exec(self) {
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
                .spawn_am_pe(
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
    table: Darc<Vec<AtomicUsize>>,
}

#[am]
impl LamellarAM for HistoAm {
    async fn exec(self) {
        for i in &self.indices {
            self.table[*i].fetch_add(1, Ordering::Relaxed);
        }
    }
}

fn lamellar_am_histogram(world: &LamellarWorld, indices: Vec<usize>) {
    let mut table = Vec::with_capacity(indices.len());
    for _ in 0..indices.len() {
        table.push(AtomicUsize::new(0));
    }
    let table = Darc::new(world, table).block().expect("failed to create Darc");
    let indices = Arc::new(indices);

    let num_threads = std::cmp::max(world.num_threads_per_pe() / 4, 1);
    let chunk_size = indices.len() / num_threads;
    world.barrier();
    let timer = std::time::Instant::now();

    for thread_id in 0..num_threads {
        let _ = world
            .spawn_am_local(HistoLaunch {
                indices: indices.clone(),
                thread_id,
                chunk_size,
                table: table.clone(),
            })
            .spawn();
    }
    world.wait_all(); // wait for every HistoLaunch (and the HistoAms it spawns) to finish
    world.barrier(); // wait for every PE to finish before reading the shared table

    println!("Lamellar AM Time: {:?}", timer.elapsed());
    println!(
        "Sum: {:?}",
        table.iter().map(|e| e.load(Ordering::SeqCst)).sum::<usize>()
    );
}

#[lamellar::main]
fn main() {
    let table_size = 1_000_000;

    let indices = generate_random_indices(table_size, table_size);
    serial_histogram(&indices);

    let world = LamellarWorldBuilder::new().build();
    let per_pe = table_size / world.num_pes();
    let pe_indices = generate_random_indices(per_pe, per_pe);

    lamellar_unsafe_histogram(&world, &pe_indices);
    world.barrier();
    lamellar_histogram(&world, &pe_indices);
    world.barrier();
    lamellar_am_histogram(&world, pe_indices);
}
