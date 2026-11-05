mod active_message;
mod graph;
mod options;
mod printer;

use lamellar::active_messaging::prelude::*;

use std::collections::HashMap;

use graph::{Graph, GraphType};
use options::TcCli;
use printer::{print_am_times, print_results};

use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Variant {
    Buffered,
    AmGroup,
    Single,
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
            #[cfg(not(feature = "benchmark-record"))]
            let _ = iteration;

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
                        benchmark_record::benchmark_report!(
                            @triangle_count,
                            &world, &cli, &graph, variant, Some(*buf_size), iteration, times,
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
                        benchmark_record::benchmark_report!(
                            @triangle_count,
                            &world, &cli, &graph, variant, Some(*buf_size), iteration, times,
                        );
                    }
                }
                Variant::Single => {
                    let times = active_message::single::triangle_count(&world, &cli, &graph);
                    variant_results.entry(0).or_insert(Vec::new()).push(times);
                    print_am_times(my_pe, &variant, 0, times);
                    benchmark_record::benchmark_report!(
                        @triangle_count,
                        &world, &cli, &graph, variant, None, iteration, times,
                    );
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
