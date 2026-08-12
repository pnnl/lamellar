mod active_message;
mod array;
mod options;
mod printer;

use array::{ArrayDistribution, ArrayType};
use options::IndexSize;
use printer::{print_am_times, print_array_times, print_results};

use clap::{Parser, ValueEnum};
use std::collections::HashMap;

#[derive(ValueEnum, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Variant {
    CasDart,
    CasDartRemote,
    PushDart,
    CasDartGroup,
    CasDartGroupRemote,
    PushDartGroup,
    BufferedCasDart,
    BufferedCasDartRemote,
    BufferedPushDart,
    UnsafeArray,
    AtomicArray,
    LocalLockArray,
}

#[lamellar::main]
fn main() {
    let world = lamellar::LamellarWorldBuilder::new()
        // .with_executor(lamellar::ExecutorType::LamellarWorkStealing)
        .build();
    let my_pe = world.my_pe();
    let num_pes = world.num_pes();
    let cli = options::RandPermCli::parse();

    let iterations = cli.iterations;

    if my_pe == 0 {
        cli.describe(num_pes);
    }

    let variants = match &cli.variants {
        Some(v) => v.clone(),
        None => vec![
            Variant::CasDart,
            Variant::CasDartRemote,
            Variant::PushDart,
            Variant::CasDartGroup,
            Variant::CasDartGroupRemote,
            Variant::PushDartGroup,
            Variant::BufferedCasDart,
            Variant::BufferedCasDartRemote,
            Variant::BufferedPushDart,
            Variant::UnsafeArray,
            Variant::AtomicArray,
            Variant::LocalLockArray,
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

    let mut results = HashMap::new();

    for variant in variants {
        let variant_results = results.entry(variant).or_insert(HashMap::new());

        for iteration in 0..iterations {
            match variant {
                Variant::CasDart => {
                    for idx_size in &am_index_size {
                        let times =
                            active_message::single_cas_am::rand_perm(&world, &cli, idx_size);
                        variant_results
                            .entry(format!("{idx_size:?}"))
                            .or_insert(Vec::new())
                            .push(times);
                        print_am_times(&cli, my_pe, &variant, idx_size, times);
                        benchmark_record::benchmark_report!(
                            @randperm,
                            &world, &cli, variant, &format!("{idx_size:?}"), iteration, times,
                        );
                    }
                }
                Variant::CasDartRemote => {
                    let times = active_message::single_cas_am_remote::rand_perm(&world, &cli);
                    variant_results
                        .entry("None".to_string())
                        .or_insert(Vec::new())
                        .push(times);
                    print_am_times(&cli, my_pe, &variant, &IndexSize::None, times);
                    benchmark_record::benchmark_report!(
                        @randperm,
                        &world, &cli, variant, "None", iteration, times,
                    );
                }
                Variant::PushDart => {
                    let times = active_message::single_push_am::rand_perm(&world, &cli);
                    variant_results
                        .entry("None".to_string())
                        .or_insert(Vec::new())
                        .push(times);
                    print_am_times(&cli, my_pe, &variant, &IndexSize::None, times);
                    benchmark_record::benchmark_report!(
                        @randperm,
                        &world, &cli, variant, "None", iteration, times,
                    );
                }
                Variant::CasDartGroup => {
                    for idx_size in &am_index_size {
                        let times = active_message::cas_am_group::rand_perm(&world, &cli, idx_size);
                        variant_results
                            .entry(format!("{idx_size:?}"))
                            .or_insert(Vec::new())
                            .push(times);
                        print_am_times(&cli, my_pe, &variant, idx_size, times);
                        benchmark_record::benchmark_report!(
                            @randperm,
                            &world, &cli, variant, &format!("{idx_size:?}"), iteration, times,
                        );
                    }
                }
                Variant::CasDartGroupRemote => {
                    let times = active_message::cas_am_group_remote::rand_perm(&world, &cli);
                    print_am_times(&cli, my_pe, &variant, &IndexSize::None, times);
                    benchmark_record::benchmark_report!(
                        @randperm,
                        &world, &cli, variant, "None", iteration, times,
                    );
                }
                Variant::PushDartGroup => {
                    let times = active_message::push_am_group::rand_perm(&world, &cli);
                    variant_results
                        .entry("None".to_string())
                        .or_insert(Vec::new())
                        .push(times);
                    print_am_times(&cli, my_pe, &variant, &IndexSize::None, times);
                    benchmark_record::benchmark_report!(
                        @randperm,
                        &world, &cli, variant, "None", iteration, times,
                    );
                }
                Variant::BufferedCasDart => {
                    for idx_size in &am_index_size {
                        let times = active_message::buffered_cas_am::rand_perm(
                            &world, &cli, false, idx_size,
                        );
                        variant_results
                            .entry(format!("{idx_size:?}"))
                            .or_insert(Vec::new())
                            .push(times);
                        print_am_times(&cli, my_pe, &variant, idx_size, times);
                        benchmark_record::benchmark_report!(
                            @randperm,
                            &world, &cli, variant, &format!("{idx_size:?}"), iteration, times,
                        );
                    }
                }
                Variant::BufferedCasDartRemote => {
                    let times = active_message::buffered_cas_am_remote::rand_perm(&world, &cli);
                    variant_results
                        .entry("None".to_string())
                        .or_insert(Vec::new())
                        .push(times);
                    print_am_times(&cli, my_pe, &variant, &IndexSize::None, times);
                    benchmark_record::benchmark_report!(
                        @randperm,
                        &world, &cli, variant, "None", iteration, times,
                    );
                }
                Variant::BufferedPushDart => {
                    let times = active_message::buffered_push_am::rand_perm(&world, &cli);
                    variant_results
                        .entry("None".to_string())
                        .or_insert(Vec::new())
                        .push(times);
                    print_am_times(&cli, my_pe, &variant, &IndexSize::None, times);
                    benchmark_record::benchmark_report!(
                        @randperm,
                        &world, &cli, variant, "None", iteration, times,
                    );
                }
                Variant::UnsafeArray => {
                    for distribution in &array_distribution {
                        let times = array::rand_perm(&world, &cli, ArrayType::Unsafe, distribution);
                        variant_results
                            .entry(format!("{distribution:?}"))
                            .or_insert(Vec::new())
                            .push(times);
                        print_array_times(&cli, my_pe, num_pes, &variant, distribution, times);
                        benchmark_record::benchmark_report!(
                            @randperm,
                            &world, &cli, variant, &format!("{distribution:?}"), iteration, times,
                        );
                    }
                }
                Variant::AtomicArray => {
                    for distribution in &array_distribution {
                        let times = array::rand_perm(&world, &cli, ArrayType::Atomic, distribution);
                        variant_results
                            .entry(format!("{distribution:?}"))
                            .or_insert(Vec::new())
                            .push(times);
                        print_array_times(&cli, my_pe, num_pes, &variant, distribution, times);
                        benchmark_record::benchmark_report!(
                            @randperm,
                            &world, &cli, variant, &format!("{distribution:?}"), iteration, times,
                        );
                    }
                }
                Variant::LocalLockArray => {
                    for distribution in &array_distribution {
                        let times =
                            array::rand_perm(&world, &cli, ArrayType::LocalLock, distribution);
                        variant_results
                            .entry(format!("{distribution:?}"))
                            .or_insert(Vec::new())
                            .push(times);
                        print_array_times(&cli, my_pe, num_pes, &variant, distribution, times);
                        benchmark_record::benchmark_report!(
                            @randperm,
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
                // &cli,
                my_pe,
                // num_pes,
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
