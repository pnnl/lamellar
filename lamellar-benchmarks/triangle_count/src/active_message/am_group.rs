use crate::{
    graph::{Graph, GraphData},
    options::TcCli,
};

use lamellar::active_messaging::prelude::*;
use lamellar::array::prelude::*;
use lamellar::darc::prelude::*;

use std::time::Duration;
#[lamellar::AmData(AmGroup)]
struct TcGroupAm {
    #[AmGroup(static)]
    graph: Darc<GraphData>, //allows us to access the graph data on other pes (with out the data explicitly being allocated in RDMA registered memory)
    node: u32,
    neighbors: Vec<u32>,
    #[AmGroup(static)]
    final_cnt: AtomicArray<usize>,
}

impl TcGroupAm {
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

#[lamellar::am(AmGroup)]
impl LamellarAM for TcGroupAm {
    async fn exec() {
        // println!("here");
        let mut cnt = 0;
        for node_1 in self
            .neighbors
            .iter()
            .filter(|n| self.graph.node_is_local(n))
        {
            //check to make sure node_1 is local to this pe
            let neighs_1 = self
                .graph
                .neighbors_iter(node_1)
                .take_while(|n| n < &node_1);
            cnt += TcGroupAm::sorted_intersection_count(self.neighbors.iter(), neighs_1);
        }
        self.final_cnt.local_data().at(0).fetch_add(cnt); //we only need to update our local portion of the count, and we know each pe only has a single element of the cnt array
    }
}

#[lamellar::AmLocalData]
struct LaunchAm {
    graph: Darc<GraphData>,
    start_node: u32,
    end_node: u32,
    final_cnt: AtomicArray<usize>,
}

#[lamellar::local_am]
impl LamellarAm for LaunchAm {
    async fn exec() {
        let mut task_group = typed_am_group!(TcGroupAm, self.graph.team());
        //nodes are striped across pes
        for node in (self.start_node..self.end_node).filter(|n| self.graph.node_is_local(n)) {
            task_group.add_am_all(TcGroupAm {
                graph: self.graph.clone(),
                node,
                neighbors: self
                    .graph
                    .neighbors_iter(&node)
                    .take_while(|n| n < &&node)
                    .copied()
                    .collect::<Vec<u32>>(), //only send neighbors that are less than node as an optimization
                final_cnt: self.final_cnt.clone(),
            });
        }
        task_group.exec().await;
    }
}

pub(crate) fn triangle_count(
    world: &LamellarWorld,
    tc_config: &TcCli,
    graph: &Graph,
    buf_size: usize,
) -> (Duration, Duration, Duration) {
    let my_pe = world.my_pe();
    let num_nodes = graph.num_nodes();

    let final_cnt = AtomicArray::new(world.team(), world.num_pes(), Distribution::Block).block();
    std::env::set_var("LAMELLAR_BATCH_OP_SIZE", format!("{}", buf_size));
    println!(
        "PE {}: starting am group triangle count with {} nodes and buffer size {}",
        my_pe, num_nodes, buf_size
    );
    world.barrier();
    let timer = std::time::Instant::now();

    let num_nodes_per_thread = num_nodes as f32 / tc_config.launch_threads as f32;
    let mut launch_tasks = vec![];
    for tid in 0..tc_config.launch_threads {
        let start_node = (tid as f32 * num_nodes_per_thread).round() as u32;
        let end_node = ((tid + 1) as f32 * num_nodes_per_thread).round() as u32;
        launch_tasks.push(
            world
                .exec_am_local(LaunchAm {
                    graph: graph.data(),
                    start_node,
                    end_node,
                    final_cnt: final_cnt.clone(),
                })
                .spawn(),
        );
    }

    //we explicitly wait for all the LaunchAMs to finish so we can explicity calculate the issue time.
    world.block_on(futures::future::join_all(launch_tasks));
    let issue_time = timer.elapsed();
    // at this point all the triangle counting active messages have been initiated.
    // calling wait_all() here will block until all the AMs including the LaunchAMs and the TcAMs have finished.
    world.wait_all(); //wait for all the triangle counting active messages to finish locally
    let local_time = timer.elapsed();
    world.barrier(); //wait for all the triangle counting active messages to finish on all PEs
    let final_cnt_sum = final_cnt.sum().block(); //reduce the final count across all PEs
    let global_time = timer.elapsed();
    if my_pe == 0 {
        println!("triangles counted: {:?}", final_cnt_sum,)
    };
    (issue_time, local_time, global_time)
}
