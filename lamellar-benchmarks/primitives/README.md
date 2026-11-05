# Celerity - Lamellar Runtime IAI Benchmarks

A comprehensive benchmarking suite for the lamellar-runtime crate using the IAI (Instruction-level Analysis and Inspection) framework.

## Overview

This crate provides detailed instruction-level benchmarks for key lamellar-runtime API functions, allowing developers to analyze the performance characteristics of distributed array operations and world management functions.

## Features

- **Instruction-level analysis** using Valgrind's Cachegrind tool
- **Comprehensive coverage** of lamellar-runtime API functions
- **Optimized benchmarks** that separate world creation from operation measurement
- **Multiple array sizes** for scalability analysis
- **Atomic operations** benchmarking (add, sub, load, store, compare_exchange, etc.)
- **Batch operations** performance testing
- **World management** function benchmarks

## Prerequisites

- Rust (latest stable version)
- Valgrind (for IAI benchmarks)

### Installing Valgrind

On Ubuntu/Debian:
```bash
sudo apt update && sudo apt install valgrind
```

On macOS:
```bash
brew install valgrind
```

## Benchmarked Functions

### World Management
- `benchmark_world_creation` - LamellarWorld creation overhead
- `benchmark_world_barrier` - Barrier synchronization
- `benchmark_world_my_pe` - PE identification
- `benchmark_world_num_pes` - PE count query

### Array Creation
- `benchmark_small_atomic_array_creation` - 1K element arrays
- `benchmark_medium_atomic_array_creation` - 100K element arrays  
- `benchmark_large_atomic_array_creation` - 1M element arrays
- `benchmark_readonly_array_creation` - ReadOnly array creation

### Atomic Operations
- `benchmark_single_atomic_add` - Single element addition
- `benchmark_batch_atomic_add` - Batch addition operations
- `benchmark_array_load` - Element loading
- `benchmark_array_store` - Element storing
- `benchmark_array_compare_swap` - Compare and exchange
- `benchmark_array_fetch_add` - Fetch and add
- `benchmark_array_sub` - Subtraction operations
- `benchmark_array_fetch_sub` - Fetch and subtract

### Array Queries
- `benchmark_array_len` - Array length query
- `benchmark_array_team` - Team information
- `benchmark_array_num_pes` - PE count for arrays
- `benchmark_readonly_array_access` - ReadOnly array access

### Complex Operations
- `benchmark_sequential_operations` - Multiple operations in sequence

## Running Benchmarks

To supress Lamellar warning about unused Active Message results during 
statistical benchmarks, set the following environment variable:

```declare -x LAMELLAR_DROPPED_UNUSED_HANDLE_WARNING=0```

This will make output much less noisy.

### Run all benchmarks:

To run instruction count (iai) profiling:

```bash
cargo bench --bench iai_lamellar_benchmark
```

To run statistical (criterion) benchmarks:

```bash
cargo bench --bench criterion_lamellar_benchmark
```



### Run specific benchmark:
```bash
cargo bench --bench iai_lamellar_benchmark benchmark_world_creation
```

```bash
cargo bench --bench iai_lamellar_benchmark array_creation
```

## Understanding Results

IAI benchmarks report instruction counts, which are deterministic and reproducible across runs. The output shows:

- **Instructions**: Total CPU instructions executed
- **Percentage changes**: Compared to previous runs (if available)

Example output:
```
benchmark_world_creation
  Instructions:             1002672 (-1.126521%)

benchmark_single_atomic_add
  Instructions:             1058180 (-20.85711%)
```

Lower instruction counts indicate better performance.

## Benchmark Design

### World Creation Optimization

The benchmarks use a singleton pattern with `OnceLock` to create the LamellarWorld once and reuse it across benchmarks. This ensures that:

1. World creation overhead is measured separately
2. Individual operation benchmarks focus on the actual operation cost
3. Results are more accurate and meaningful

### Array Size Variations

Three different array sizes are tested:
- **Small (1K)**: Fast operations, good for micro-benchmarks
- **Medium (100K)**: Realistic workload sizes
- **Large (1M)**: Stress testing and scalability analysis

## Dependencies

- `lamellar = "0.7.1"` - The distributed runtime being benchmarked
- `iai = "0.1"` - Instruction-level benchmarking framework
- `rand = "0.8"` - Random number generation for test data
- `criterion = "0.7"' - Statistical benchmarking framework

NOTE: In Cargo.toml there is a specific fork of iai used that depends on a specific
version of Valgrind for compability.  This will be updated in the future.

## Contributing

When adding new benchmarks:

1. Follow the existing naming convention: `benchmark_<operation_name>`
2. Use `black_box()` to prevent compiler optimizations
3. Use `get_world()` to access the shared world instance
4. Add the function to the `iai::main!()` macro
5. Document the benchmark purpose and expected behavior

## License

Copyright 2025 Battelle Memorial Institute

See LICENSE file in this directory
