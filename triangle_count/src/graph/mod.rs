#![allow(clippy::let_underscore_future)]

use lamellar::active_messaging::prelude::*;
use lamellar::darc::prelude::*;
use lamellar::memregion::prelude::*;
use std::error::Error;
use std::path::Path;
use std::sync::Arc;
// use std::marker::PhantomData;

use std::collections::HashMap;
use std::collections::HashSet;

use std::fs::File;
// use std::io::Write;
use std::io::{BufRead, BufReader, BufWriter};

#[allow(dead_code)]
pub mod mapgraph;
use mapgraph::{MapGraph, MapGraphIter};

// pub trait Element:
//     'static
//     + std::fmt::Debug
//     + Clone
//     + Send
//     + Sync
//     + serde::ser::Serialize
//     + for<'de> serde::Deserialize<'de>
// {
// }
// impl<
//         T: 'static
//             + std::fmt::Debug
//             + Clone
//             + Send
//             + Sync
//             + serde::ser::Serialize
//             + for<'de> serde::Deserialize<'de>,
//     > Element for T
// {
// }

#[derive(Debug, serde::Deserialize, Eq, PartialEq)]
struct Edge {
    e0: u32,
    e1: u32,
}

#[derive(Clone)]
enum EdgeList {
    Vec(Vec<u32>),
    Set(HashSet<u32>),
}

impl EdgeList {
    fn len(&self) -> usize {
        match self {
            EdgeList::Vec(vec) => vec.len(),
            EdgeList::Set(set) => set.len(),
        }
    }
    fn push(&mut self, val: u32) {
        match self {
            EdgeList::Vec(vec) => vec.push(val),
            EdgeList::Set(set) => {
                set.insert(val);
            }
        }
    }
    fn iter(&self) -> Box<dyn Iterator<Item = &u32> + '_> {
        match self {
            EdgeList::Vec(vec) => Box::new(vec.iter()),
            EdgeList::Set(set) => Box::new(set.iter()),
        }
    }
}

trait GraphOps {
    async fn get_local_neighbors(
        &self,
        neighbors: OneSidedMemoryRegion<u32>,
    ) -> OneSidedMemoryRegion<u32>;
    fn add_remote_neighbors(&mut self, node: u32, neighbors: OneSidedMemoryRegion<u32>);
    fn neighbors(&self, node: &u32) -> std::slice::Iter<'_, u32>;
    // fn lamellar_neighbors(&self, node: &u32) -> OneSidedMemoryRegion<u32>;
    fn num_nodes(&self) -> usize;
    fn node_is_local(&self, node: &u32) -> bool;
}

pub enum GraphType {
    MapGraph,
}

pub enum GraphData {
    MapGraph(MapGraph),
}

#[allow(dead_code)]
pub enum GraphIter<'a> {
    MapGraph(MapGraphIter<'a>),
}

impl<'a> Iterator for GraphIter<'a> {
    type Item = &'a u32;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            GraphIter::MapGraph(iter) => iter.next(),
        }
    }
}

impl GraphOps for GraphData {
    async fn get_local_neighbors(
        &self,
        neighbors: OneSidedMemoryRegion<u32>,
    ) -> OneSidedMemoryRegion<u32> {
        match self {
            GraphData::MapGraph(graph) => graph.get_local_neighbors(neighbors).await,
        }
    }
    fn add_remote_neighbors(&mut self, node: u32, neighbors: OneSidedMemoryRegion<u32>) {
        match self {
            GraphData::MapGraph(graph) => graph.add_remote_neighbors(node, neighbors),
        }
    }
    fn neighbors(&self, node: &u32) -> std::slice::Iter<'_, u32> {
        match self {
            GraphData::MapGraph(graph) => graph.neighbors(node),
        }
    }
    // fn lamellar_neighbors(&self, node: &u32) -> OneSidedMemoryRegion<u32> {
    //     match self {
    //         GraphData::MapGraph(graph) => graph.lamellar_neighbors(node),
    //     }
    // }
    fn num_nodes(&self) -> usize {
        match self {
            GraphData::MapGraph(graph) => graph.num_nodes(),
        }
    }
    fn node_is_local(&self, node: &u32) -> bool {
        match self {
            GraphData::MapGraph(graph) => graph.node_is_local(node),
        }
    }
}

#[allow(dead_code)]
impl GraphData {
    pub fn iter(&self) -> GraphIter<'_> {
        match self {
            GraphData::MapGraph(graph) => GraphIter::MapGraph(graph.iter()),
        }
    }
    pub fn neighbors_iter(&self, node: &u32) -> std::slice::Iter<'_, u32> {
        match self {
            GraphData::MapGraph(graph) => graph.neighbors(node),
        }
    }
    // pub fn local_neighbors(&self, node: &u32) -> OneSidedMemoryRegion<u32> {
    //     match self {
    //         GraphData::MapGraph(graph) => graph.lamellar_neighbors(node),
    //     }
    // }
    pub fn node_is_local(&self, node: &u32) -> bool {
        match self {
            GraphData::MapGraph(graph) => graph.node_is_local(node),
        }
    }
}

#[lamellar::AmLocalData]
struct RelabelMapAm {
    start_index: usize,
    nodes: Vec<u32>,
    relabeled: OneSidedMemoryRegion<u32>,
}
#[lamellar::local_am]
impl LamellarAM for RelabelMapAm {
    async fn exec() {
        let relabled = unsafe { self.relabeled.as_mut_slice() };
        for (i, node) in self.nodes.iter().enumerate() {
            relabled[*node as usize] = (i + self.start_index) as u32;
        }
    }
}

#[lamellar::AmLocalData]
struct RelabelAm {
    nodes: Vec<(EdgeList, OneSidedMemoryRegion<u32>, usize)>,
    relabeled: OneSidedMemoryRegion<u32>,
}
#[lamellar::local_am]
impl LamellarAM for RelabelAm {
    async fn exec() {
        let relabled = unsafe { self.relabeled.as_slice() };
        for nodes in &self.nodes {
            let old_nodes = &nodes.0;
            let new_nodes = unsafe { nodes.1.as_mut_slice() };
            if old_nodes.len() == 0 {
                new_nodes[0] = (self.relabeled.len() + 1) as u32;
            } else {
                // for i in 0..old_nodes.len() {
                for (i, old_node) in old_nodes.iter().enumerate() {
                    new_nodes[i] = relabled[*old_node as usize];
                }
                new_nodes.sort_unstable();
            }
        }
    }
}

#[lamellar::AmData]
struct LocalNeighborsAM {
    graph: LocalRwDarc<GraphData>,
    node_and_neighbors: Vec<(u32, OneSidedMemoryRegion<u32>)>,
}

#[lamellar::am]
impl LamellarAM for LocalNeighborsAM {
    async fn exec() {
        let graph = self.graph.read().await;
        let mut nodes = Vec::with_capacity(self.node_and_neighbors.len());
        let mut remotes = Vec::with_capacity(self.node_and_neighbors.len());
        for (node, neighbors) in &self.node_and_neighbors {
            nodes.push(*node);
            remotes.push(graph.get_local_neighbors(neighbors.clone()));
        }
        let remotes = lamellar::world.join_all(remotes).await;
        // for (node, neighbors) in nodes.iter().zip(remotes.iter()) {
        //     graph.add_remote_neighbors(*node, neighbors.clone());
        // }
        let _ = lamellar::world
            .exec_am_all(RemoteNeighborsAM {
                graph: self.graph.clone(),
                nodes,
                neighbors: remotes,
            })
            .spawn();
    }
}

#[lamellar::AmData]
struct RemoteNeighborsAM {
    graph: LocalRwDarc<GraphData>,
    nodes: Vec<u32>,
    neighbors: Vec<OneSidedMemoryRegion<u32>>,
}
#[lamellar::am]
impl LamellarAM for RemoteNeighborsAM {
    async fn exec() {
        let mut graph = self.graph.write().await;
        for (node, neighbors) in self.nodes.iter().zip(self.neighbors.iter()) {
            graph.add_remote_neighbors(*node, neighbors.clone());
        }
    }
}

#[derive(Clone)]
pub struct Graph {
    graph: Darc<GraphData>,
    world: LamellarWorld,
    pub my_pe: usize,
}

#[allow(dead_code)]
impl Graph {
    pub fn new(fpath: &str, graph_type: GraphType, world: LamellarWorld) -> Graph {
        let my_pe = world.my_pe();
        let _ = graph_type;
        let graph = GraphData::MapGraph(MapGraph::new(world.team().clone()));
        let graph = LocalRwDarc::new(world.team(), graph).block().unwrap(); // we are creating with the world team so should be valid on all pes

        Graph::load(fpath, &world, &graph).expect("error reading graph");
        if my_pe == 0 {
            println!("Done loading graph!");
        }
        let g = Graph {
            world,
            graph: graph.into_darc().block(),
            my_pe,
        };
        if g.my_pe == 0 {
            println!("Done creating graph!");
        }
        g.barrier();
        g
    }

    fn load(
        fpath: &str,
        world: &LamellarWorld,
        graph: &LocalRwDarc<GraphData>,
    ) -> Result<(), Box<dyn Error>> {
        if world.my_pe() == 0 && Graph::parse(fpath, b'\t', world, graph).is_err() {
            Graph::parse(fpath, b' ', world, graph)?;
        }
        world.barrier();
        Ok(())
    }

    fn parse(
        fpath: &str,
        delim: u8,
        world: &LamellarWorld,
        graph: &LocalRwDarc<GraphData>,
    ) -> Result<usize, Box<dyn Error>> {
        let path = Path::new(&fpath);

        let mut cur_node = 1;
        let mut num_edges: usize = 0;
        let mut num_nodes = 0;

        let start = std::time::Instant::now();
        let mut indices;
        let mut temp_neighbor_list: Vec<EdgeList>;
        match path.extension().unwrap().to_str().unwrap() {
            "bin" => {
                let file = File::open(path)?;
                let mut rdr = BufReader::new(file);
                num_nodes = bincode::deserialize_from(&mut rdr).unwrap();
                temp_neighbor_list = vec![EdgeList::Vec(Vec::new()); num_nodes];
                while let Ok(node) = bincode::deserialize_from::<_, u32>(&mut rdr) {
                    temp_neighbor_list[node as usize] =
                        EdgeList::Vec(bincode::deserialize_from::<_, Vec<u32>>(&mut rdr).unwrap());
                    num_edges += temp_neighbor_list[node as usize].len();
                    if node % 1000000 == 0 {
                        println!("{:?} nodes loaded", node);
                    }
                }
                indices = (0..num_nodes).collect::<Vec<_>>();
            }
            "mm" => {
                let file = File::open(path)?;
                let rdr = BufReader::new(file);
                let mut lines = rdr
                    .lines()
                    .map(|l| l.unwrap())
                    .skip_while(|l| l.starts_with("%"));
                let line = lines.next().unwrap();
                // println!("header {line}");
                let vals = line.split_whitespace().collect::<Vec<_>>();
                assert_eq!(vals[0], vals[1]);
                num_nodes = vals[0].parse().unwrap();
                num_edges = vals[2].parse().unwrap();

                temp_neighbor_list = vec![EdgeList::Set(HashSet::new()); num_nodes];

                for line in lines {
                    let vals = line.split_whitespace().collect::<Vec<_>>();
                    let e0: usize = vals[0].parse::<usize>().unwrap() - 1;
                    let e1: usize = vals[1].parse::<usize>().unwrap() - 1;
                    temp_neighbor_list[e0].push(e1 as u32);
                    temp_neighbor_list[e1].push(e0 as u32);
                    if cur_node % 1000000 == 0 {
                        println!("{:?} nodes loaded", cur_node);
                    }
                    cur_node += 1;
                }
                indices = (0..num_nodes).collect::<Vec<_>>();
                indices.sort_by_key(|&i| -(temp_neighbor_list[i].len() as isize));
                //would be nice to do this multithreaded
            }
            "tsv" => {
                let mut rdr = csv::ReaderBuilder::new()
                    .has_headers(false)
                    .delimiter(delim)
                    .from_path(path)?;
                let mut edges = EdgeList::Set(HashSet::new());
                temp_neighbor_list = vec![];
                for result in rdr.deserialize() {
                    let edge: Edge = result?;
                    if cur_node != edge.e1 {
                        num_edges += edges.len();
                        temp_neighbor_list.push(edges);

                        cur_node = edge.e1;
                        num_nodes += 1;
                        edges = EdgeList::Set(HashSet::new());
                        if cur_node % 100000 == 0 {
                            println!("{:?} nodes loaded", cur_node);
                        }
                    }
                    edges.push(edge.e0 - 1);
                }

                num_edges += edges.len();
                temp_neighbor_list.push(edges);
                num_nodes += 1;
                indices = (0..num_nodes).collect::<Vec<_>>();
                indices.sort_by_key(|&i| -(temp_neighbor_list[i].len() as isize));
                //would be nice to do this multithreaded
            }
            _ => {
                panic!("unhandled file format");
            }
        }
        println!("read time: {:?}", start.elapsed().as_secs_f64());
        println!("{num_nodes}");

        let start = std::time::Instant::now();

        // let mut indices = (0..num_nodes).collect::<Vec<_>>();

        println!("ind len {}", indices.len());

        let relabeled = world.alloc_one_sided_mem_region::<u32>(num_nodes);
        let relabeled_slice = unsafe { relabeled.as_mut_slice() };

        let mut cnt = 0;
        for (i, node) in indices.iter().enumerate() {
            //would be nice to do this multithreaded
            relabeled_slice[*node] = i as u32;
            cnt += temp_neighbor_list[i].len();
        }

        println!("num_edges {} {}", num_edges, cnt);
        println!("reorder map time: {:?}", start.elapsed().as_secs_f64());

        let mut temp_nodes = vec![];
        let mut neigh_list = vec![];
        let mut size = 0;
        for (i, nodes) in temp_neighbor_list.drain(..).enumerate() {
            if size > num_edges / 10 {
                let _ = world
                    .exec_am_local(RelabelAm {
                        nodes: temp_nodes,
                        relabeled: relabeled.clone(),
                    })
                    .spawn();
                temp_nodes = vec![];
                size = 0;
            }
            let nodes_len = nodes.len();
            size += nodes_len;
            let temp = world.alloc_one_sided_mem_region::<u32>(std::cmp::max(nodes_len, 1));
            neigh_list.push(temp.clone());
            temp_nodes.push((nodes, temp, i));
        }
        if size > 0 {
            let _ = world
                .exec_am_local(RelabelAm {
                    nodes: temp_nodes,
                    relabeled: relabeled.clone(),
                })
                .spawn();
        }
        println!("reorder issue time: {:?}", start.elapsed().as_secs_f64());
        world.wait_all();
        println!("reorder time: {:?}", start.elapsed().as_secs_f64());

        let task_group = LamellarTaskGroup::new(world.team());
        let mut pe_neigh_lists: HashMap<usize, Vec<(u32, OneSidedMemoryRegion<u32>)>> =
            HashMap::new();
        for pe in 0..world.num_pes() {
            pe_neigh_lists.insert(pe, vec![]);
        }
        for (old_node, region) in neigh_list.iter().enumerate() {
            let new_node = unsafe { relabeled.as_slice()[old_node] as usize };
            let pe = new_node % world.num_pes();
            pe_neigh_lists
                .get_mut(&pe)
                .unwrap()
                .push((new_node as u32, region.clone()));
        }

        let num_batches = std::cmp::min(10, neigh_list.len()).max(1);
        for (pe, neigh_lists) in pe_neigh_lists.iter_mut() {
            let batch_size = neigh_lists.len().div_ceil(num_batches).max(1);

            while neigh_lists.len() > batch_size {
                let _ = task_group
                    .exec_am_pe(
                        *pe,
                        LocalNeighborsAM {
                            graph: graph.clone(),
                            node_and_neighbors: neigh_lists
                                .split_off(neigh_lists.len() - batch_size),
                        },
                    )
                    .spawn();
            }
            if !neigh_lists.is_empty() {
                let _ = task_group
                    .exec_am_pe(
                        *pe,
                        LocalNeighborsAM {
                            graph: graph.clone(),
                            node_and_neighbors: neigh_lists.clone(),
                        },
                    )
                    .spawn();
            }
        }

        println!("distribute issue time: {:?}", start.elapsed().as_secs_f64());
        task_group.wait_all();
        world.wait_all();
        println!("distribute time: {:?}", start.elapsed().as_secs_f64());
        Ok(num_nodes)
    }

    pub fn data(&self) -> Darc<GraphData> {
        self.graph.clone()
    }

    pub fn neighbors(&self, node: &u32) -> std::slice::Iter<'_, u32> {
        self.graph.neighbors(node)
    }

    pub fn iter(&self) -> GraphIter<'_> {
        self.graph.iter()
    }

    pub fn barrier(&self) {
        self.world.barrier();
    }

    pub fn team(&self) -> Arc<LamellarTeam> {
        self.world.team()
    }

    pub fn num_pes(&self) -> usize {
        self.world.num_pes()
    }

    pub fn my_pe(&self) -> usize {
        self.world.my_pe()
    }

    pub fn num_nodes(&self) -> usize {
        self.graph.num_nodes()
    }

    pub fn node_is_local(&self, node: &u32) -> bool {
        //probably should abstract this out to the graphops trait
        *node as usize % self.num_pes() == self.my_pe()
    }

    pub fn dump_to_bin(&self, name: &str) {
        let mut file = BufWriter::new(File::create(name).expect("error dumping graph"));
        bincode::serialize_into(&mut file, &self.num_nodes()).unwrap();
        for n0 in (0..self.num_nodes()).map(|n| n as u32) {
            if self.node_is_local(&n0) {
                let neighs = self
                    .graph
                    .neighbors_iter(&n0)
                    .take_while(|n| n < &&n0)
                    .collect::<Vec<_>>();
                if !neighs.is_empty() {
                    bincode::serialize_into(&mut file, &n0).unwrap();
                    bincode::serialize_into(&mut file, &neighs).unwrap();
                }
            }
        }
    }

    pub fn print(&self) {
        for n0 in (0..self.num_nodes()).map(|n| n as u32) {
            let neighs = if self.node_is_local(&n0) {
                self.graph
                    .neighbors_iter(&n0)
                    // .take_while(|n| n < &&n0)
                    .collect::<Vec<_>>()
            } else {
                vec![]
            };
            println!("{}: {:?} {:?}", n0, neighs.len(), neighs);
        }
    }
}
