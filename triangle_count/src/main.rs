mod active_message;
mod graph;
mod options;
mod printer;

use lamellar::active_messaging::prelude::*;

use std::collections::HashMap;

use graph::{Graph, GraphType};
use options::TcCli;
use printer::{print_am_times, print_results};

use benchmark_record::{embed_build_time_info, BenchmarkInformation};
use clap::{Parser, ValueEnum};
use std::time::Duration;

#[derive(ValueEnum, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Variant {
    Buffered,
    AmGroup,
    Single,
}

fn record_run(
    world: &lamellar::LamellarWorld,
    cli: &TcCli,
    graph: &Graph,
    variant: Variant,
    buffer_size: Option<usize>,
    iteration: usize,
    times: (Duration, Duration, Duration),
) {
    if world.my_pe() != 0 {
        return;
    }

    let global_secs = times.2.as_secs_f64();
    let mut record = BenchmarkInformation::new();
    embed_build_time_info!(record);
    record.with_output("variant", format!("{variant:?}"));
    record.with_output(
        "buffer_size",
        buffer_size.map_or_else(|| "none".to_owned(), |size| size.to_string()),
    );
    record.with_output("iteration", iteration.to_string());
    record.with_output("graph_file", cli.graph_file.clone());
    record.with_output("num_nodes", graph.num_nodes().to_string());
    record.with_output("num_pes", world.num_pes().to_string());
    record.with_output("launch_threads", cli.launch_threads.to_string());
    record.with_output("issue_time_secs", times.0.as_secs_f64().to_string());
    record.with_output("local_completion_secs", times.1.as_secs_f64().to_string());
    record.with_output("global_completion_secs", global_secs.to_string());
    record.with_output("mb_sent_total", world.MB_sent().to_string());
    if let Err(error) = record.write(&record.default_output_path("benchmarking")) {
        eprintln!("failed to write benchmark record: {error}");
    }
}

#[lamellar::main]
fn main() {
    let world = lamellar::LamellarWorldBuilder::new().build();
    let my_pe = world.my_pe();
    let cli = TcCli::parse();

    let iterations = cli.iterations;

    if my_pe == 0 {
        cli.describe();
    }

    let file = &cli.graph_file;
    let graph: Graph = Graph::new(file, GraphType::MapGraph, world.clone());

    let variants = match &cli.variants {
        Some(v) => v.clone(),
        None => vec![Variant::Buffered, Variant::AmGroup, Variant::Single],
    };

    let mut results = HashMap::new();

    for variant in variants {
        let variant_results = results.entry(variant).or_insert(HashMap::new());
        for iteration in 0..iterations {
            match variant {
                Variant::Buffered => {
                    for buf_size in &cli.buffer_size {
                        let times = active_message::buffered::triangle_count(
                            &world, &cli, &graph, *buf_size,
                        );
                        variant_results
                            .entry(*buf_size)
                            .or_insert(Vec::new())
                            .push(times);
                        print_am_times(my_pe, &variant, *buf_size, times);
                        record_run(
                            &world,
                            &cli,
                            &graph,
                            variant,
                            Some(*buf_size),
                            iteration,
                            times,
                        );
                    }
                }
                Variant::AmGroup => {
                    for buf_size in &cli.buffer_size {
                        let times = active_message::am_group::triangle_count(
                            &world, &cli, &graph, *buf_size,
                        );
                        variant_results
                            .entry(*buf_size)
                            .or_insert(Vec::new())
                            .push(times);
                        print_am_times(my_pe, &variant, *buf_size, times);
                        record_run(
                            &world,
                            &cli,
                            &graph,
                            variant,
                            Some(*buf_size),
                            iteration,
                            times,
                        );
                    }
                }
                Variant::Single => {
                    let times = active_message::single::triangle_count(&world, &cli, &graph);
                    variant_results.entry(0).or_insert(Vec::new()).push(times);
                    print_am_times(my_pe, &variant, 0, times);
                    record_run(&world, &cli, &graph, variant, None, iteration, times);
                }
            }
        }
    }
    let max_buf_len = cli
        .buffer_size
        .iter()
        .map(|e| format!("{e}").len())
        .max()
        .unwrap();
    for (variant, variant_results) in &results {
        for (sub_variant, times) in variant_results {
            print_results(
                // &cli,
                my_pe,
                // num_pes,
                &format!("{:<1$}", format!("{variant:?}"), cli.max_variant_len()),
                &format!("{:<1$}", format!("{sub_variant}"), max_buf_len),
                times,
            )
        }
    }
    world.wait_all();
    world.barrier();
    // if world.my_pe() == 0{
    //     graph.print();
    // }
}
