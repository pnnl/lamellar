use lamellar::memregion::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

// use parking_lot::RwLock;

use super::GraphOps;
// use crate::Element;

pub struct MapGraph {
    team: Arc<LamellarTeam>,
    neighbors: HashMap<u32, OneSidedMemoryRegion<u32>>,
    // num_nodes: usize,
}
pub struct MapGraphIter<'a> {
    iter: std::collections::hash_map::Keys<'a, u32, OneSidedMemoryRegion<u32>>,
}
// pub struct MapGraphRangeIter<'a>{
//     neighbors: &'a HashMap<u32,OneSidedMemoryRegion<u32>>,
//     cur: usize,
//     end: usize,
// }

// struct DistributeNeighborsAM{
//     node: u32,
//     neighbors: OneSidedMemoryRegion<u32>
// }

impl MapGraph {
    pub fn new(team: Arc<LamellarTeam>) -> MapGraph {
        MapGraph {
            team,
            neighbors: HashMap::new(),
            // num_nodes: 0,
        }
    }
    pub fn iter(&self) -> MapGraphIter<'_> {
        MapGraphIter {
            iter: self.neighbors.keys(),
        }
    }
    // pub fn range_iter(&self,start:usize, end:usize) -> MapGraphRangeIter<'_> {
    //     MapGraphRangeIter{
    //         neighbors: &self.neighbors,
    //         cur: start,
    //         end: end,
    //     }
    // }
}

impl GraphOps for MapGraph {
    async fn get_local_neighbors(
        &self,
        neighbors: OneSidedMemoryRegion<u32>,
        dest: OneSidedMemoryRegion<u32>,
    ) -> OneSidedMemoryRegion<u32> {
        // `dest` is a slice of one batch-wide region allocated by the caller
        // (see LocalNeighborsAM::exec), not a per-node allocation, to avoid one
        // RDMA registration per node. Other slices of the same region exist, but
        // they cover disjoint ranges and nothing else touches this one until it
        // is returned, so writing into it here is safe.
        let mut lmr_neighbors: LamellarBuffer<u32, _> = dest.into();
        unsafe {
            if neighbors.len() > 0 {
                neighbors
                    .get_into_buffer(0, lmr_neighbors.split_off(0))
                    .await;
            }
        }
        let lmr_neighbors = lmr_neighbors.async_unwrap().await;
        // self.neighbors.insert(node, lmr_neighbors.clone());
        lmr_neighbors
    }
    fn add_remote_neighbors(&mut self, node: u32, neighbors: OneSidedMemoryRegion<u32>) {
        self.neighbors.insert(node, neighbors);
    }
    fn neighbors(&self, node: &u32) -> std::slice::Iter<'_, u32> {
        if let Some(n) = self.neighbors.get(node) {
            unsafe { n.as_slice().iter() }
        } else {
            panic!("node {:?} does not exist in graph", node);
        }
    }
    // fn lamellar_neighbors(&self, node: &u32) -> OneSidedMemoryRegion<u32> {
    //     if let Some(n) = self.neighbors.get(node) {
    //         n.clone()
    //     } else {
    //         panic!("node {:?} does not exist in graph", node);
    //     }
    // }

    fn num_nodes(&self) -> usize {
        self.neighbors.len()
    }

    fn node_is_local(&self, node: &u32) -> bool {
        //probably should abstract this out to the graphops trait
        *node as usize % self.team.num_pes() == self.team.team_pe_id().unwrap()
    }
}

impl<'a> Iterator for MapGraphIter<'a> {
    type Item = &'a u32;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

// impl <'a> Iterator for MapGraphRangeIter<'a> {
//     type Item = &'a u32;
//     fn next(&mut self) -> Option<Self::Item> {
//         if self.cur == self.end{
//             None
//         }
//         else{
//             let cur = self.cur as u32;
//             self.cur+=1;
//             self.neighbors.get(&cur).map(|x| x)
//         }
//     }
// }

impl Drop for MapGraph {
    fn drop(&mut self) {
        for (_node, lmr) in self.neighbors.drain() {
            drop(lmr)
        }
    }
}
