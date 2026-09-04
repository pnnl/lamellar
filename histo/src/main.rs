mod active_message;
mod array;
mod options;
mod printer;

use array::{ArrayDistribution, ArrayType};
use options::{HistoCli, IndexSize};
use printer::{print_am_times, print_array_times, print_results};

use clap::{Parser, ValueEnum};
use rand::prelude::*;
use std::collections::BTreeMap;
use std::sync::Arc;

// #[global_allocator]
// static ALLOC: dhat::Alloc = dhat::Alloc;
// use tikv_jemallocator::Jemalloc;
// #[global_allocator]
// static GLOBAL: Jemalloc = Jemalloc;

#[derive(ValueEnum, Debug, Clone, Copy, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub enum Variant {
    TestAm,
    UnsafeAM,
    SafeAm,
    UnsafeBufferedAm,
    SafeBufferedAm,
    UnsafeAmGroup,
    SafeAmGroup,
    UnsafeArray,
    AtomicArray,
    LocalLockArray,
    UnsafeArrayCustomType,
    AtomicArrayCustomType,
    LocalLockArrayCustomType,
}

#[lamellar::main]
fn main() {
    let world = lamellar::LamellarWorldBuilder::new().build();
    let my_pe = world.my_pe();
    let num_pes = world.num_pes();
    let cli = HistoCli::parse();

    let global_count = cli.total_table_size(num_pes);
    let l_num_updates = cli.pe_updates(num_pes);
    let iterations = cli.iterations;

    if my_pe == 0 {
        cli.describe(num_pes);
    }

    let variants = match &cli.variants {
        Some(v) => v.clone(),
        None => vec![
            Variant::UnsafeAM,
            Variant::SafeAm,
            Variant::UnsafeBufferedAm,
            Variant::SafeBufferedAm,
            Variant::UnsafeAmGroup,
            Variant::SafeAmGroup,
            Variant::UnsafeArray,
            Variant::AtomicArray,
            Variant::LocalLockArray,
            Variant::UnsafeArrayCustomType,
            Variant::AtomicArrayCustomType,
            Variant::LocalLockArrayCustomType,
        ],
    };

    let am_index_size = match &cli.am_index_size {
        Some(v) => v.clone(),
        None => vec![IndexSize::U32, IndexSize::Usize],
    };

    let array_distribution = match &cli.array_distribution {
        Some(v) => v.clone(),
        None => vec![ArrayDistribution::Block, ArrayDistribution::Cyclic],
    };

    let mut rng: StdRng = SeedableRng::seed_from_u64(my_pe as u64);

    let mut results = BTreeMap::new();

    for variant in variants {
        let variant_results = results.entry(variant).or_insert(BTreeMap::new());

        for iteration in 0..iterations {
            #[cfg(not(feature = "benchmark-record"))]
            let _ = iteration;

            // create new random indicies for each iteration
            let rand_index = Arc::new(
                (0..l_num_updates)
                    .map(|_| rng.gen_range(0, global_count))
                    .collect::<Vec<usize>>(),
            );

            match variant {
                Variant::TestAm => {
                    // for idx_size in &am_index_size {
                    //     let times = active_message::test_am::histo(
                    //         &world,
                    //         &cli,
                    //         &rand_index,
                    //         true,
                    //         idx_size,
                    //     );
                    //     variant_results
                    //         .entry(format!("{idx_size:?}"))
                    //         .or_insert(Vec::new())
                    //         .push(times.clone());
                    //     print_am_times(&cli, my_pe, num_pes, &variant, &idx_size, times);
                    // }
                }
                Variant::UnsafeAM => {
                    for idx_size in &am_index_size {
                        let times =
                            active_message::am::histo(&world, &cli, &rand_index, false, idx_size);
                        variant_results
                            .entry(format!("{idx_size:?}"))
                            .or_insert(Vec::new())
                            .push(times);
                        print_am_times(&cli, my_pe, num_pes, &variant, idx_size, times);
                        benchmark_record::benchmark_report!(
                            @update_table,
                            &world, &cli, variant, &format!("{idx_size:?}"), iteration, times,
                        );
                    }
                }
                Variant::SafeAm => {
                    for idx_size in &am_index_size {
                        let times =
                            active_message::am::histo(&world, &cli, &rand_index, true, idx_size);
                        variant_results
                            .entry(format!("{idx_size:?}"))
                            .or_insert(Vec::new())
                            .push(times);
                        print_am_times(&cli, my_pe, num_pes, &variant, idx_size, times);
                        benchmark_record::benchmark_report!(
                            @update_table,
                            &world, &cli, variant, &format!("{idx_size:?}"), iteration, times,
                        );
                    }
                }
                Variant::UnsafeBufferedAm => {
                    for idx_size in &am_index_size {
                        let times = active_message::buffered_am::histo(
                            &world,
                            &cli,
                            &rand_index,
                            false,
                            idx_size,
                        );
                        variant_results
                            .entry(format!("{idx_size:?}"))
                            .or_insert(Vec::new())
                            .push(times);
                        print_am_times(&cli, my_pe, num_pes, &variant, idx_size, times);
                        benchmark_record::benchmark_report!(
                            @update_table,
                            &world, &cli, variant, &format!("{idx_size:?}"), iteration, times,
                        );
                    }
                }
                Variant::SafeBufferedAm => {
                    for idx_size in &am_index_size {
                        let times = active_message::buffered_am::histo(
                            &world,
                            &cli,
                            &rand_index,
                            true,
                            idx_size,
                        );
                        variant_results
                            .entry(format!("{idx_size:?}"))
                            .or_insert(Vec::new())
                            .push(times);
                        print_am_times(&cli, my_pe, num_pes, &variant, idx_size, times);
                        benchmark_record::benchmark_report!(
                            @update_table,
                            &world, &cli, variant, &format!("{idx_size:?}"), iteration, times,
                        );
                    }
                }
                Variant::UnsafeAmGroup => {
                    for idx_size in &am_index_size {
                        let times = active_message::am_group::histo(
                            &world,
                            &cli,
                            &rand_index,
                            false,
                            idx_size,
                        );
                        variant_results
                            .entry(format!("{idx_size:?}"))
                            .or_insert(Vec::new())
                            .push(times);
                        print_am_times(&cli, my_pe, num_pes, &variant, idx_size, times);
                        benchmark_record::benchmark_report!(
                            @update_table,
                            &world, &cli, variant, &format!("{idx_size:?}"), iteration, times,
                        );
                    }
                }
                Variant::SafeAmGroup => {
                    for idx_size in &am_index_size {
                        let times = active_message::am_group::histo(
                            &world,
                            &cli,
                            &rand_index,
                            true,
                            idx_size,
                        );
                        variant_results
                            .entry(format!("{idx_size:?}"))
                            .or_insert(Vec::new())
                            .push(times);
                        print_am_times(&cli, my_pe, num_pes, &variant, idx_size, times);
                        benchmark_record::benchmark_report!(
                            @update_table,
                            &world, &cli, variant, &format!("{idx_size:?}"), iteration, times,
                        );
                    }
                }
                Variant::UnsafeArray => {
                    for distribution in &array_distribution {
                        let times = array::histo(
                            &world,
                            &cli,
                            &rand_index,
                            ArrayType::Unsafe,
                            distribution,
                        );
                        variant_results
                            .entry(format!("{distribution:?}"))
                            .or_insert(Vec::new())
                            .push(times);
                        print_array_times(&cli, my_pe, num_pes, &variant, distribution, times);
                        benchmark_record::benchmark_report!(
                            @update_table,
                            &world, &cli, variant, &format!("{distribution:?}"), iteration, times,
                        );
                    }
                }
                Variant::AtomicArray => {
                    for distribution in &array_distribution {
                        let times = array::histo(
                            &world,
                            &cli,
                            &rand_index,
                            ArrayType::Atomic,
                            distribution,
                        );
                        variant_results
                            .entry(format!("{distribution:?}"))
                            .or_insert(Vec::new())
                            .push(times);
                        print_array_times(&cli, my_pe, num_pes, &variant, distribution, times);
                        benchmark_record::benchmark_report!(
                            @update_table,
                            &world, &cli, variant, &format!("{distribution:?}"), iteration, times,
                        );
                    }
                }
                Variant::LocalLockArray => {
                    for distribution in &array_distribution {
                        let times = array::histo(
                            &world,
                            &cli,
                            &rand_index,
                            ArrayType::LocalLock,
                            distribution,
                        );
                        variant_results
                            .entry(format!("{distribution:?}"))
                            .or_insert(Vec::new())
                            .push(times);
                        print_array_times(&cli, my_pe, num_pes, &variant, distribution, times);
                        benchmark_record::benchmark_report!(
                            @update_table,
                            &world, &cli, variant, &format!("{distribution:?}"), iteration, times,
                        );
                    }
                }
                Variant::UnsafeArrayCustomType => {
                    for distribution in &array_distribution {
                        let times = array::histo_custom_type(
                            &world,
                            &cli,
                            &rand_index,
                            ArrayType::Unsafe,
                            distribution,
                        );
                        variant_results
                            .entry(format!("{distribution:?}"))
                            .or_insert(Vec::new())
                            .push(times);
                        print_array_times(&cli, my_pe, num_pes, &variant, distribution, times);
                        benchmark_record::benchmark_report!(
                            @update_table,
                            &world, &cli, variant, &format!("{distribution:?}"), iteration, times,
                        );
                    }
                }
                Variant::AtomicArrayCustomType => {
                    for distribution in &array_distribution {
                        let times = array::histo_custom_type(
                            &world,
                            &cli,
                            &rand_index,
                            ArrayType::Atomic,
                            distribution,
                        );
                        variant_results
                            .entry(format!("{distribution:?}"))
                            .or_insert(Vec::new())
                            .push(times);
                        print_array_times(&cli, my_pe, num_pes, &variant, distribution, times);
                        benchmark_record::benchmark_report!(
                            @update_table,
                            &world, &cli, variant, &format!("{distribution:?}"), iteration, times,
                        );
                    }
                }
                Variant::LocalLockArrayCustomType => {
                    for distribution in &array_distribution {
                        let times = array::histo_custom_type(
                            &world,
                            &cli,
                            &rand_index,
                            ArrayType::LocalLock,
                            distribution,
                        );
                        variant_results
                            .entry(format!("{distribution:?}"))
                            .or_insert(Vec::new())
                            .push(times);
                        print_array_times(&cli, my_pe, num_pes, &variant, distribution, times);
                        benchmark_record::benchmark_report!(
                            @update_table,
                            &world, &cli, variant, &format!("{distribution:?}"), iteration, times,
                        );
                    }
                }
            }
        }
    }

    for (variant, variant_results) in results {
        for (sub_variant, times) in variant_results {
            print_results(
                &cli,
                my_pe,
                num_pes,
                &format!("{:<1$}", format!("{variant:?}"), cli.max_variant_len()),
                &format!(
                    "{:<1$}",
                    format!("{sub_variant}"),
                    cli.max_index_size_len() + cli.max_array_distribution_len()
                ),
                &times,
            )
        }
    }
}
