use benchmark_record::{embed_build_time_info, BenchmarkInformation};
use lamellar::active_messaging::prelude::*;
use lamellar::array::prelude::*;
use lamellar::darc::prelude::*;
use lamellar_graph::{Graph, GraphData, GraphType};
use std::time::Instant;

#[lamellar::AmLocalData]
struct LaunchAm {
    graph: Graph,
    start: u32,
    end: u32,
    final_cnt: AtomicArray<usize>,
    buf_size: usize,
}

#[lamellar::local_am]
impl LamellarAM for LaunchAm {
    async fn exec() {
        let task_group = LamellarTaskGroup::new(lamellar::world.clone());
        let graph_data = self.graph.data();
        let mut buffer = vec![];
        let mut cur_len = 0;
        for node_0 in (self.start..self.end).filter(|n| self.graph.node_is_local(n)) {
            let neighbors = graph_data
                .neighbors_iter(&node_0)
                .take_while(|n| n < &&node_0)
                .copied()
                .collect::<Vec<u32>>();
            cur_len += neighbors.len();
            buffer.push((node_0, neighbors));
            if cur_len > self.buf_size {
                let _ = task_group
                    .exec_am_all(BufferedTcAm {
                        graph: graph_data.clone(),
                        data: buffer,
                        final_cnt: self.final_cnt.clone(),
                    })
                    .spawn();
                buffer = vec![];
                cur_len = 0;
            }
        }
        if cur_len > 0 {
            let _ = task_group
                .exec_am_all(BufferedTcAm {
                    graph: graph_data,
                    data: buffer,
                    final_cnt: self.final_cnt.clone(),
                })
                .spawn();
        }
        task_group.await_all().await;
    }
}

#[lamellar::AmData]
struct BufferedTcAm {
    graph: Darc<GraphData>,
    data: Vec<(u32, Vec<u32>)>,
    final_cnt: AtomicArray<usize>,
}

impl BufferedTcAm {
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
impl LamellarAM for BufferedTcAm {
    async fn exec() {
        let mut cnt = 0;
        for (_node_0, neighbors) in &self.data {
            for node_1 in neighbors.iter().filter(|n| self.graph.node_is_local(n)) {
                let neighs_1 = self
                    .graph
                    .neighbors_iter(node_1)
                    .take_while(|n| n < &node_1);
                cnt += BufferedTcAm::sorted_intersection_count(neighbors.iter(), neighs_1);
            }
        }
        self.final_cnt.local_data().at(0).fetch_add(cnt);
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
    let graph: Graph = Graph::new(file, GraphType::MapGraph, world.clone());
    let final_cnt = AtomicArray::new(world.team(), world.num_pes(), Distribution::Block).block();
    let batch_size = graph.num_nodes() as f32 / launch_threads as f32;

    for buf_size in [10_usize, 100, 1_000, 10_000, 100_000, 1_000_000] {
        let mut bench = BenchmarkInformation::new();
        embed_build_time_info!(bench);
        bench.with_output("num_pes", world.num_pes().to_string());
        bench.with_output("launch_threads", launch_threads.to_string());
        bench.with_output("buffer_size", buf_size.to_string());
        bench.with_output("num_nodes", graph.num_nodes().to_string());
        world.barrier();
        let timer = Instant::now();
        let mut reqs = vec![];
        for tid in 0..launch_threads {
            let start = (tid as f32 * batch_size).round() as u32;
            let end = ((tid + 1) as f32 * batch_size).round() as u32;
            reqs.push(
                world
                    .exec_am_local(LaunchAm {
                        graph: graph.clone(),
                        start,
                        end,
                        final_cnt: final_cnt.clone(),
                        buf_size,
                    })
                    .spawn(),
            );
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
        let triangle_count = final_cnt.sum().block();
        let global_secs = timer.elapsed().as_secs_f64();
        bench.with_output("global_time_secs", global_secs.to_string());
        bench.with_output("triangles_counted", format!("{triangle_count:?}"));
        bench.with_output("mb_sent_total", world.MB_sent().to_string());
        if my_pe == 0 {
            bench.write(&bench.default_output_path("benchmarking"));
        }
        final_cnt.dist_iter().for_each(|x| x.store(0)).block();
    }
}
