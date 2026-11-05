use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use lamellar::active_messaging::prelude::*;
use lamellar::array::prelude::*;
use rand::{distributions::Distribution, rngs::StdRng, SeedableRng};
use std::hint::black_box;
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

// Helper function to create atomic arrays of different sizes
fn create_atomic_array(world: &LamellarWorld, size: usize) -> AtomicArray<usize> {
    AtomicArray::<usize>::new(world, size, lamellar::Distribution::Block).block()
}

fn create_read_only_array(world: &LamellarWorld, size: usize) -> ReadOnlyArray<usize> {
    ReadOnlyArray::new(world, size, lamellar::Distribution::Block).block()
}


// Benchmark array creation with different sizes
fn benchmark_array_creation(c: &mut Criterion) {
    let world = get_world();
    let mut group = c.benchmark_group("array_creation");
    
    for &size in &[SMALL_ARRAY_SIZE, MEDIUM_ARRAY_SIZE, LARGE_ARRAY_SIZE] {
        group.bench_with_input(BenchmarkId::new("atomic", size), &size, |b, &size| {
            b.iter(|| {
                let _array = black_box(create_atomic_array(world, size));
            })
        });
        
        group.bench_with_input(BenchmarkId::new("readonly", size), &size, |b, &size| {
            b.iter(|| {
                let _array = black_box(create_read_only_array(world, size));
            })
        });
    }
    group.finish();
}

// Benchmark atomic operations
fn benchmark_atomic_operations(c: &mut Criterion) {
    let world = get_world();
    let mut group = c.benchmark_group("atomic_operations");
    
    // Single atomic add
    group.bench_function("single_add", |b| {
        let array = create_atomic_array(world, SMALL_ARRAY_SIZE);
        b.iter(|| {
            let _result = black_box(array.add(0, 1).block());
        })
    });
    
    // Batch atomic add
    group.bench_function("batch_add", |b| {
        let array = create_atomic_array(world, MEDIUM_ARRAY_SIZE);
        let mut rng: StdRng = SeedableRng::seed_from_u64(42);
        let range = rand::distributions::Uniform::new(0, MEDIUM_ARRAY_SIZE);
        let indices: Vec<usize> = range.sample_iter(&mut rng).take(NUM_UPDATES).collect();
        
        b.iter(|| {
            let _result = black_box(array.batch_add(&indices, 1).block());
        })
    });
    
    // Load operation
    group.bench_function("load", |b| {
        let array = create_atomic_array(world, SMALL_ARRAY_SIZE);
        b.iter(|| {
            let _result = black_box(array.load(0).block());
        })
    });
    
    // Store operation
    group.bench_function("store", |b| {
        let array = create_atomic_array(world, SMALL_ARRAY_SIZE);
        b.iter(|| {
            let _result = black_box(array.store(0, 42).block());
        })
    });
    
    // Compare and swap
    group.bench_function("compare_exchange", |b| {
        let array = create_atomic_array(world, SMALL_ARRAY_SIZE);
        b.iter(|| {
            let _result = black_box(array.compare_exchange(0, 0, 1).block());
        })
    });
    
    // Fetch and add
    group.bench_function("fetch_add", |b| {
        let array = create_atomic_array(world, SMALL_ARRAY_SIZE);
        b.iter(|| {
            let _result = black_box(array.fetch_add(0, 1).block());
        })
    });
    
    // Subtraction operations
    group.bench_function("sub", |b| {
        let array = create_atomic_array(world, SMALL_ARRAY_SIZE);
        b.iter(|| {
            let _result = black_box(array.sub(0, 1).block());
        })
    });
    
    // Fetch and subtract
    group.bench_function("fetch_sub", |b| {
        let array = create_atomic_array(world, SMALL_ARRAY_SIZE);
        b.iter(|| {
            let _result = black_box(array.fetch_sub(0, 1).block());
        })
    });
    
    group.finish();
}

// Benchmark array query operations
fn benchmark_array_queries(c: &mut Criterion) {
    let world = get_world();
    let mut group = c.benchmark_group("array_queries");
    
    // Array length
    group.bench_function("len", |b| {
        let array = create_atomic_array(world, MEDIUM_ARRAY_SIZE);
        b.iter(|| {
            let _len = black_box(array.len());
        })
    });
    
    // Array team
    group.bench_function("team", |b| {
        let array = create_atomic_array(world, MEDIUM_ARRAY_SIZE);
        b.iter(|| {
            let _team = black_box(array.team());
        })
    });
    
    // Array num_pes
    group.bench_function("num_pes", |b| {
        let array = create_atomic_array(world, SMALL_ARRAY_SIZE);
        b.iter(|| {
            let _num_pes = black_box(array.num_pes());
        })
    });
    
    // ReadOnly array access
    group.bench_function("readonly_access", |b| {
        let array = create_read_only_array(world, SMALL_ARRAY_SIZE);
        b.iter(|| {
            let _result = black_box(array.at(0));
        })
    });
    
    group.finish();
}

// Benchmark world operations
fn benchmark_world_operations(c: &mut Criterion) {
    let world = get_world();
    let mut group = c.benchmark_group("world_operations");
    
    // World barrier
    group.bench_function("barrier", |b| {
        b.iter(|| {
            black_box(world.barrier());
        })
    });
    
    // World my_pe
    group.bench_function("my_pe", |b| {
        b.iter(|| {
            let _my_pe = black_box(world.my_pe());
        })
    });
    
    // World num_pes
    group.bench_function("num_pes", |b| {
        b.iter(|| {
            let _num_pes = black_box(world.num_pes());
        })
    });
    
    group.finish();
}

// Benchmark sequential operations
fn benchmark_sequential_operations(c: &mut Criterion) {
    let world = get_world();
    
    c.bench_function("sequential_operations", |b| {
        let array = create_atomic_array(world, SMALL_ARRAY_SIZE);
        b.iter(|| {
            // Perform a sequence of operations
            let _store = black_box(array.store(0, 10).block());
            let _load = black_box(array.load(0).block());
            let _add = black_box(array.add(0, 5).block());
            let _fetch_add = black_box(array.fetch_add(0, 3).block());
        })
    });
}

// Benchmark scalability with different array sizes
fn benchmark_scalability(c: &mut Criterion) {
    let world = get_world();
    let mut group = c.benchmark_group("scalability");
    
    for &size in &[1000, 10000, 100000, 1000000] {
        // Array creation scalability
        group.bench_with_input(BenchmarkId::new("array_creation", size), &size, |b, &size| {
            b.iter(|| {
                let _array = black_box(create_atomic_array(world, size));
            })
        });
        
        // Single operation scalability (using different indices to avoid cache effects)
        group.bench_with_input(BenchmarkId::new("single_add", size), &size, |b, &size| {
            let array = create_atomic_array(world, size);
            let index = size / 2; // Use middle index
            b.iter(|| {
                let _result = black_box(array.add(index, 1).block());
            })
        });
    }
    
    group.finish();
}

// Benchmark batch operations with different batch sizes
fn benchmark_batch_operations(c: &mut Criterion) {
    let world = get_world();
    let mut group = c.benchmark_group("batch_operations");
    
    let array = create_atomic_array(world, MEDIUM_ARRAY_SIZE);
    
    for &batch_size in &[10, 100, 1000, 10000] {
        group.bench_with_input(BenchmarkId::new("batch_add", batch_size), &batch_size, |b, &batch_size| {
            let mut rng: StdRng = SeedableRng::seed_from_u64(42);
            let range = rand::distributions::Uniform::new(0, MEDIUM_ARRAY_SIZE);
            let indices: Vec<usize> = range.sample_iter(&mut rng).take(batch_size).collect();
            
            b.iter(|| {
                let _result = black_box(array.batch_add(&indices, 1).block());
            })
        });
    }
    
    group.finish();
}

// Benchmark memory access patterns
fn benchmark_memory_patterns(c: &mut Criterion) {
    let world = get_world();
    let mut group = c.benchmark_group("memory_patterns");
    
    let array = create_atomic_array(world, MEDIUM_ARRAY_SIZE);
    
    // Sequential access
    group.bench_function("sequential_access", |b| {
        b.iter(|| {
            for i in 0..100 {
                let _result = black_box(array.load(i).block());
            }
        })
    });
    
    // Random access
    group.bench_function("random_access", |b| {
        let mut rng: StdRng = SeedableRng::seed_from_u64(42);
        let range = rand::distributions::Uniform::new(0, MEDIUM_ARRAY_SIZE);
        let indices: Vec<usize> = range.sample_iter(&mut rng).take(100).collect();
        
        b.iter(|| {
            for &index in &indices {
                let _result = black_box(array.load(index).block());
            }
        })
    });
    
    // Strided access
    group.bench_function("strided_access", |b| {
        b.iter(|| {
            for i in (0..10000).step_by(100) {
                let _result = black_box(array.load(i).block());
            }
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,

    benchmark_array_creation,
    benchmark_atomic_operations,
    benchmark_array_queries,
    benchmark_world_operations,
    benchmark_sequential_operations,
    benchmark_scalability,
    benchmark_batch_operations,
    benchmark_memory_patterns
);

criterion_main!(benches);
