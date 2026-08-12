mod active_message;
mod array;
mod options;
mod printer;

use array::{ArrayDistribution, ArrayType};
use options::{IndexGatherCli, IndexSize};
use printer::{print_am_times, print_array_times, print_results};

use benchmark_record::{embed_build_time_info, BenchmarkInformation};
use clap::{Parser, ValueEnum};
use rand::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

#[derive(ValueEnum, Debug, Clone, Copy, Hash, PartialEq, Eq)]
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
    ReadOnlyArray,
    LocalLockArray,
}

fn record_run(
    world: &lamellar::LamellarWorld,
    cli: &IndexGatherCli,
    variant: Variant,
    subvariant: &str,
    iteration: usize,
    times: (Duration, Duration, Duration, Duration),
) {
    if world.my_pe() != 0 {
        return;
    }

    let num_pes = world.num_pes();
    let global_secs = times.3.as_secs_f64();
    let mut record = BenchmarkInformation::new();
    embed_build_time_info!(record);
    record.with_output("variant", format!("{variant:?}"));
    record.with_output("subvariant", subvariant.to_owned());
    record.with_output("iteration", iteration.to_string());
    record.with_output("num_pes", num_pes.to_string());
    record.with_output("updates_total", cli.total_updates(num_pes).to_string());
    record.with_output("updates_per_pe", cli.pe_updates(num_pes).to_string());
    record.with_output(
        "table_size_total",
        cli.total_table_size(num_pes).to_string(),
    );
    record.with_output("table_size_per_pe", cli.pe_table_size(num_pes).to_string());
    record.with_output("launch_threads", cli.launch_threads.to_string());
    record.with_output("buffer_size", cli.buffer_size.to_string());
    record.with_output("issue_time_secs", times.0.as_secs_f64().to_string());
    record.with_output("launch_completion_secs", times.1.as_secs_f64().to_string());
    record.with_output("local_completion_secs", times.2.as_secs_f64().to_string());
    record.with_output("global_completion_secs", global_secs.to_string());
    record.with_output(
        "global_mups",
        ((cli.total_updates(num_pes) as f64 / 1_000_000.0) / global_secs.max(f64::MIN_POSITIVE))
            .to_string(),
    );
    record.with_output("mb_sent_total", world.MB_sent().to_string());
    if let Err(error) = record.write(&record.default_output_path("benchmarking")) {
        eprintln!("failed to write benchmark record: {error}");
    }
}

#[lamellar::main]
fn main() {
    let world = lamellar::LamellarWorldBuilder::new().build();
    let my_pe = world.my_pe();
    let num_pes = world.num_pes();
    let cli = IndexGatherCli::parse();

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

    let mut results = HashMap::new();

    for variant in variants {
        let variant_results = results.entry(variant).or_insert(HashMap::new());

        for iteration in 0..iterations {
            // create new random indicies for each iteration
            let rand_index = Arc::new(
                (0..l_num_updates)
                    .map(|_| rng.gen_range(0, global_count))
                    .collect::<Vec<usize>>(),
            );

            match variant {
                Variant::TestAm => {
                    for idx_size in &am_index_size {
                        let times = active_message::test_am::index_gather(
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
                        record_run(
                            &world,
                            &cli,
                            variant,
                            &format!("{idx_size:?}"),
                            iteration,
                            times,
                        );
                    }
                }
                Variant::UnsafeAM => {
                    for idx_size in &am_index_size {
                        let times = active_message::am::index_gather(
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
                        record_run(
                            &world,
                            &cli,
                            variant,
                            &format!("{idx_size:?}"),
                            iteration,
                            times,
                        );
                    }
                }
                Variant::SafeAm => {
                    for idx_size in &am_index_size {
                        let times = active_message::am::index_gather(
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
                        record_run(
                            &world,
                            &cli,
                            variant,
                            &format!("{idx_size:?}"),
                            iteration,
                            times,
                        );
                    }
                }
                Variant::UnsafeBufferedAm => {
                    for idx_size in &am_index_size {
                        let times = active_message::buffered_am::index_gather(
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
                        record_run(
                            &world,
                            &cli,
                            variant,
                            &format!("{idx_size:?}"),
                            iteration,
                            times,
                        );
                    }
                }
                Variant::SafeBufferedAm => {
                    for idx_size in &am_index_size {
                        let times = active_message::buffered_am::index_gather(
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
                        record_run(
                            &world,
                            &cli,
                            variant,
                            &format!("{idx_size:?}"),
                            iteration,
                            times,
                        );
                    }
                }
                Variant::UnsafeAmGroup => {
                    for idx_size in &am_index_size {
                        let times = active_message::am_group::index_gather(
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
                        record_run(
                            &world,
                            &cli,
                            variant,
                            &format!("{idx_size:?}"),
                            iteration,
                            times,
                        );
                    }
                }
                Variant::SafeAmGroup => {
                    for idx_size in &am_index_size {
                        let times = active_message::am_group::index_gather(
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
                        record_run(
                            &world,
                            &cli,
                            variant,
                            &format!("{idx_size:?}"),
                            iteration,
                            times,
                        );
                    }
                }
                Variant::UnsafeArray => {
                    for distribution in &array_distribution {
                        let times = array::index_gather(
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
                        record_run(
                            &world,
                            &cli,
                            variant,
                            &format!("{distribution:?}"),
                            iteration,
                            times,
                        );
                    }
                }
                Variant::ReadOnlyArray => {
                    for distribution in &array_distribution {
                        let times = array::index_gather(
                            &world,
                            &cli,
                            &rand_index,
                            ArrayType::ReadOnly,
                            distribution,
                        );
                        variant_results
                            .entry(format!("{distribution:?}"))
                            .or_insert(Vec::new())
                            .push(times);
                        print_array_times(&cli, my_pe, num_pes, &variant, distribution, times);
                        record_run(
                            &world,
                            &cli,
                            variant,
                            &format!("{distribution:?}"),
                            iteration,
                            times,
                        );
                    }
                }
                Variant::AtomicArray => {
                    for distribution in &array_distribution {
                        let times = array::index_gather(
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
                        record_run(
                            &world,
                            &cli,
                            variant,
                            &format!("{distribution:?}"),
                            iteration,
                            times,
                        );
                    }
                }
                Variant::LocalLockArray => {
                    for distribution in &array_distribution {
                        let times = array::index_gather(
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
                        record_run(
                            &world,
                            &cli,
                            variant,
                            &format!("{distribution:?}"),
                            iteration,
                            times,
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
