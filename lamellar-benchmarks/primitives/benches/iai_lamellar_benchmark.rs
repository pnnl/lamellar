use iai::black_box;
use lamellar::active_messaging::prelude::*;
use lamellar::array::prelude::*;
use rand::{distributions::Distribution, rngs::StdRng, SeedableRng};
use std::sync::OnceLock;

// Test constants
const SMALL_ARRAY_SIZE: usize = 1000;
const MEDIUM_ARRAY_SIZE: usize = 100000;
const LARGE_ARRAY_SIZE: usize = 1000000;
const NUM_UPDATES: usize = 1000;

// Global world instance - created once and reused
static WORLD: OnceLock<LamellarWorld> = OnceLock::new();

// Helper function to get or create the world
fn get_world() -> &'static LamellarWorld {
    WORLD.get_or_init(|| {
        lamellar::LamellarWorldBuilder::new().build()
    })
}

// Test world creation separately
fn benchmark_world_creation() {
    let _world = black_box(lamellar::LamellarWorldBuilder::new().build());
}

// Helper function to create atomic arrays of different sizes
fn create_atomic_array(world: &LamellarWorld, size: usize) -> AtomicArray<usize> {
    AtomicArray::<usize>::new(world, size, lamellar::Distribution::Block).block()
}

fn create_read_only_array(world: &LamellarWorld, size: usize) -> ReadOnlyArray<usize> {
    ReadOnlyArray::new(world, size, lamellar::Distribution::Block).block()
}

// Benchmark: Small AtomicArray creation
fn benchmark_small_atomic_array_creation() {
    let world = get_world();
    let _array = black_box(create_atomic_array(world, SMALL_ARRAY_SIZE));
}

// Benchmark: Medium AtomicArray creation
fn benchmark_medium_atomic_array_creation() {
    let world = get_world();
    let _array = black_box(create_atomic_array(world, MEDIUM_ARRAY_SIZE));
}

// Benchmark: Large AtomicArray creation
fn benchmark_large_atomic_array_creation() {
    let world = get_world();
    let _array = black_box(create_atomic_array(world, LARGE_ARRAY_SIZE));
}

// Benchmark: ReadOnlyArray creation
fn benchmark_readonly_array_creation() {
    let world = get_world();
    let _array = black_box(create_read_only_array(world, MEDIUM_ARRAY_SIZE));
}

// Benchmark: Single atomic add operation
fn benchmark_single_atomic_add() {
    let world = get_world();
    let array = create_atomic_array(world, SMALL_ARRAY_SIZE);
    
    let _result = black_box(array.add(0, 1).block());
}

// Benchmark: Batch atomic add operations
fn benchmark_batch_atomic_add() {
    let world = get_world();
    let array = create_atomic_array(world, MEDIUM_ARRAY_SIZE);
    
    let mut rng: StdRng = SeedableRng::seed_from_u64(42);
    let range = rand::distributions::Uniform::new(0, MEDIUM_ARRAY_SIZE);
    let indices: Vec<usize> = range.sample_iter(&mut rng).take(NUM_UPDATES).collect();
    
    let _result = black_box(array.batch_add(&indices, 1).block());
}

// Benchmark: Array load operation
fn benchmark_array_load() {
    let world = get_world();
    let array = create_atomic_array(world, SMALL_ARRAY_SIZE);
    
    let _result = black_box(array.load(0).block());
}

// Benchmark: Array store operation
fn benchmark_array_store() {
    let world = get_world();
    let array = create_atomic_array(world, SMALL_ARRAY_SIZE);
    
    let _result = black_box(array.store(0, 42).block());
}

// Benchmark: Array compare and swap
fn benchmark_array_compare_swap() {
    let world = get_world();
    let array = create_atomic_array(world, SMALL_ARRAY_SIZE);
    
    let _result = black_box(array.compare_exchange(0, 0, 1).block());
}

// Benchmark: Array fetch and add
fn benchmark_array_fetch_add() {
    let world = get_world();
    let array = create_atomic_array(world, SMALL_ARRAY_SIZE);
    
    let _result = black_box(array.fetch_add(0, 1).block());
}

// Benchmark: ReadOnlyArray access
fn benchmark_readonly_array_access() {
    let world = get_world();
    let array = create_read_only_array(world, SMALL_ARRAY_SIZE);
    
    let _result = black_box(array.at(0));
}

// Benchmark: World barrier synchronization
fn benchmark_world_barrier() {
    let world = get_world();
    black_box(world.barrier());
}

// Benchmark: Array length operation
fn benchmark_array_len() {
    let world = get_world();
    let array = create_atomic_array(world, MEDIUM_ARRAY_SIZE);
    
    let _len = black_box(array.len());
}

// Benchmark: Array team query
fn benchmark_array_team() {
    let world = get_world();
    let array = create_atomic_array(world, MEDIUM_ARRAY_SIZE);
    
    let _team = black_box(array.team());
}

// Benchmark: Array num_pes query
fn benchmark_array_num_pes() {
    let world = get_world();
    let array = create_atomic_array(world, SMALL_ARRAY_SIZE);
    
    let _num_pes = black_box(array.num_pes());
}

// Benchmark: Multiple array operations in sequence
fn benchmark_sequential_operations() {
    let world = get_world();
    let array = create_atomic_array(world, SMALL_ARRAY_SIZE);
    
    // Perform a sequence of operations
    let _store = black_box(array.store(0, 10).block());
    let _load = black_box(array.load(0).block());
    let _add = black_box(array.add(0, 5).block());
    let _fetch_add = black_box(array.fetch_add(0, 3).block());
}

// Benchmark: World PE information
fn benchmark_world_my_pe() {
    let world = get_world();
    let _my_pe = black_box(world.my_pe());
}

// Benchmark: World num_pes information
fn benchmark_world_num_pes() {
    let world = get_world();
    let _num_pes = black_box(world.num_pes());
}

// Benchmark: Array sub operation
fn benchmark_array_sub() {
    let world = get_world();
    let array = create_atomic_array(world, SMALL_ARRAY_SIZE);
    
    let _result = black_box(array.sub(0, 1).block());
}

// Benchmark: Array fetch_sub operation
fn benchmark_array_fetch_sub() {
    let world = get_world();
    let array = create_atomic_array(world, SMALL_ARRAY_SIZE);
    
    let _result = black_box(array.fetch_sub(0, 1).block());
}

iai::main!(
    benchmark_world_creation,
    benchmark_small_atomic_array_creation,
    benchmark_medium_atomic_array_creation,
    benchmark_large_atomic_array_creation,
    benchmark_readonly_array_creation,
    benchmark_single_atomic_add,
    benchmark_batch_atomic_add,
    benchmark_array_load,
    benchmark_array_store,
    benchmark_array_compare_swap,
    benchmark_array_fetch_add,
    benchmark_readonly_array_access,
    benchmark_world_barrier,
    benchmark_array_len,
    benchmark_array_team,
    benchmark_array_num_pes,
    benchmark_sequential_operations,
    benchmark_world_my_pe,
    benchmark_world_num_pes,
    benchmark_array_sub,
    benchmark_array_fetch_sub
);
