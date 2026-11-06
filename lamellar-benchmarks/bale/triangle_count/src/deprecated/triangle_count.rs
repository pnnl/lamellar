use benchmark_record::{embed_build_time_info, BenchmarkInformation};
use lamellar::active_messaging::prelude::*;
use lamellar::darc::prelude::*;
use lamellar_graph::{Graph, GraphData, GraphType};
use std::sync::atomic::{AtomicUsize, Ordering};

#[lamellar::AmData]
struct CntAm {
    final_cnt: Darc<AtomicUsize>,
    cnt: usize,
}

#[lamellar::am]
impl LamellarAM for CntAm {
    async fn exec() {
        self.final_cnt.fetch_add(self.cnt, Ordering::Relaxed);
    }
}

#[lamellar::AmLocalData]
struct LaunchAm {
    graph: Graph,
    start: u32,
    end: u32,
    final_cnt: Darc<AtomicUsize>,
}

#[lamellar::local_am]
impl LamellarAM for LaunchAm {
    async fn exec() {
        let task_group = LamellarTaskGroup::new(lamellar::world.clone());
        let graph_data = self.graph.data();
        for node_0 in (self.start..self.end).filter(|n| self.graph.node_is_local(n)) {
            let _ = task_group
                .exec_am_all(TcAm {
                    graph: graph_data.clone(),
                    node: node_0,
                    neighbors: graph_data
                        .neighbors_iter(&node_0)
                        .take_while(|n| n < &&node_0)
                        .copied()
                        .collect::<Vec<u32>>(),
                    final_cnt: self.final_cnt.clone(),
                })
                .spawn();
        }
        task_group.await_all().await;
    }
}

#[lamellar::AmData]
struct TcAm {
    graph: Darc<GraphData>,
    node: u32,
    neighbors: Vec<u32>,
    final_cnt: Darc<AtomicUsize>,
}

impl TcAm {
    fn sorted_intersection_count<'a>(
        set0: impl Iterator<Item = &'a u32> + Clone,
        mut set1: impl Iterator<Item = &'a u32> + Clone,
    ) -> usize {
        let mut count = 0;
        if let Some(mut node_1) = set1.next() {
            for node_0 in set0 {
                while node_1 < node_0 {
                    node_1 = match set1.next() {
                        Some(node_1) => node_1,
                        None => return count,
                    };
                }
                if node_0 == node_1 {
                    count += 1;
                }
            }
        }
        count
    }
}

#[lamellar::am]
impl LamellarAM for TcAm {
    async fn exec() {
        let mut cnt = 0;
        for node_1 in self
            .neighbors
            .iter()
            .filter(|n| self.graph.node_is_local(n))
        {
            let neighs_1 = self
                .graph
                .neighbors_iter(node_1)
                .take_while(|n| n < &node_1);
            cnt += TcAm::sorted_intersection_count(self.neighbors.iter(), neighs_1);
        }
        self.final_cnt.fetch_add(cnt, Ordering::SeqCst);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let file = &args[1];
    let launch_threads = args
        .get(2)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or_else(|| match std::env::var("LAMELLAR_THREADS") {
            Ok(n) => n.parse::<usize>().unwrap(),
            Err(_) => 1,
        });

    let world = lamellar::LamellarWorldBuilder::new().build();
    let my_pe = world.my_pe();
    let num_pes = world.num_pes();
    let mut bench = BenchmarkInformation::new();
    embed_build_time_info!(bench);
    bench.with_output("my_pe", my_pe.to_string());
    bench.with_output("num_pes", num_pes.to_string());

    let graph: Graph = Graph::new(file, GraphType::MapGraph, world.clone());
    graph.dump_to_bin(&format!("{file}.bin"));
    let final_cnt = Darc::new(&world, AtomicUsize::new(0)).block().unwrap();

    world.barrier();
    let timer = std::time::Instant::now();
    let batch_size = graph.num_nodes() as f32 / launch_threads as f32;
    let mut reqs = vec![];
    for tid in 0..launch_threads {
        let start = (tid as f32 * batch_size).round() as u32;
        let end = ((tid + 1) as f32 * batch_size).round() as u32;
        reqs.push(world.exec_am_local(LaunchAm {
            graph: graph.clone(),
            start,
            end,
            final_cnt: final_cnt.clone(),
        }));
    }
    world.block_on(async move {
        for req in reqs {
            req.await;
        }
    });
    bench.with_output("issue_time_secs", timer.elapsed().as_secs_f64().to_string());
    world.wait_all();
    bench.with_output("local_time_secs", timer.elapsed().as_secs_f64().to_string());
    world.barrier();

    if my_pe != 0 {
        world.block_on(world.exec_am_pe(
            0,
            CntAm {
                final_cnt: final_cnt.clone(),
                cnt: final_cnt.load(Ordering::SeqCst),
            },
        ));
    }
    world.barrier();
    bench.with_output(
        "triangles_counted",
        final_cnt.load(Ordering::SeqCst).to_string(),
    );
    bench.with_output(
        "global_time_secs",
        timer.elapsed().as_secs_f64().to_string(),
    );
    if my_pe == 0 {
        bench.write(&bench.default_output_path("benchmarking"));
    }
}
