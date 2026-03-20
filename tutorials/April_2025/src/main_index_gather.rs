

use rand::prelude::*; // prelude imports contain common types and traits for the given crate
use rayon::prelude::*; 

use std::sync::{atomic::{AtomicUsize, Ordering}, Arc};

use lamellar::array::prelude::*;
use lamellar::active_messaging::prelude::*;


fn generate_random_vec(n: usize, max_val: usize) -> Vec<usize> {
    let mut rng = rand::rng();
    (0..n)
        .map(|_| rng.random_range(0..max_val))
        .collect()
}  

fn serial_indexgather(indices: &[usize], table: &[usize]) {
    let timer = std::time::Instant::now();
    let results = indices.iter().map(|i| table[*i]).collect::<Vec<usize>>();
    println!("Serial Time: {:?}", timer.elapsed());
    
}

fn rayon_indexgather(indices: &[usize], table: &[usize]) {
    let mut table = indices.iter().map(|i| *i *2).collect::<Vec<usize>>();
    let mut results = vec![0;indices.len()];
    // let table = Arc::new(table);
    let timer = std::time::Instant::now();
    results.par_iter_mut().enumerate().for_each(|(i,r)| {
        *r = table[indices[i]];
    }); 
    println!("Rayon Time: {:?}", timer.elapsed());
}

fn lamellar_indexgather(world: &LamellarWorld, indices: &[usize], table: &[usize]){
    let table_array = LocalLockArray::<usize>::new(world,indices.len()*world.num_pes(),lamellar::Distribution::Cyclic).block();
    table_array.write_local_data().block().copy_from_slice(table);
    let table = table_array.into_read_only().block();
    let timer = std::time::Instant::now();
    let results = table.batch_load(indices).block();
    table.barrier();
    println!("Lamellar Time: {:?}", timer.elapsed());
}

fn lamellar_am_indexgather_one_pe(world: &LamellarWorld, indices: Vec<usize>, table: Vec<usize>){
    let table = Arc::new(table);
    let indices = Arc::new(indices);
    let mut results = Vec::with_capacity(indices.len());
    let num_threads = world.num_threads_per_pe();
    let chunk_size = indices.len()/num_threads;
    let timer = std::time::Instant::now();    
    let tasks =  (0..num_threads).map(|thread_id| {
        world.exec_am_local(IndexGatherOne{indices: indices.clone(), thread_id: thread_id, chunk_size: chunk_size, table: table.clone()}).spawn()
    }).collect::<Vec<_>>();
    world.wait_all();
    println!("Lamellar Time: {:?} ", timer.elapsed());
    for task in tasks{
        let res = task.block();
        results.extend_from_slice(&res);
    }
    println!("Lamellar Time: {:?} ", timer.elapsed());

}

#[AmLocalData]
struct IndexGatherOne{
    indices: Arc::<Vec<usize>>,
    thread_id: usize,
    chunk_size: usize,
    table: Arc<Vec<usize>>,
}

#[local_am]
impl LamellarAM for IndexGatherOne{
    async fn exec(&self) -> Vec<usize>{
        let mut results = Vec::with_capacity(self.chunk_size);
        for i in &self.indices[self.thread_id*self.chunk_size..(self.thread_id+1)*self.chunk_size]{
            results.push(self.table[*i]);
        }
        results
    }
}

fn main() {
    let table_size = 1000000000;
    // let indices = generate_random_vec(table_size,table_size);
    // let table = generate_random_vec(table_size,table_size);
    // serial_indexgather(&indices, &table);
    // rayon_indexgather(&indices, &table);

    let world = LamellarWorldBuilder::new().build();
    let indices = generate_random_vec(table_size/world.num_pes(),table_size);
    let table = generate_random_vec(table_size/world.num_pes(),table_size);
    if world.my_pe() == 0 && world.num_pes() == 1{
        serial_indexgather(&indices,&table);
    }
    lamellar_indexgather(&world,&indices,&table);
    if world.num_pes() == 1{
        lamellar_am_indexgather_one_pe(&world, indices,table);
    }

}
 