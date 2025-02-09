use crate::graph::Graph;
use std::{collections::HashMap, u64};
use wg_internal::{
    network::{NodeId, SourceRoutingHeader},
    packet::FloodResponse,
};

pub struct RoutingHandler {
    graph: Graph,
    old_graph: Graph,
    current_flood_id: u64,
    // ack, nack
    pdr: HashMap<NodeId, (f32, f32)>,
    congestion: HashMap<NodeId, f32>,
}

impl RoutingHandler {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            old_graph: Graph::new(),
            current_flood_id: u64::MAX,
            pdr: HashMap::new(),
            congestion: HashMap::new(),
        }
    }

    /// Update the graph with the new flood response
    ///
    /// Description:
    ///
    /// The function create and update the graph with the fllod response.
    /// If the flood id is different from the current flood id a new graph is created with the old weights.
    pub fn update_graph(&mut self, flood: FloodResponse) {
        if self.current_flood_id != flood.flood_id {
            self.old_graph = self.graph.clone();
            self.graph.clear();
            self.current_flood_id = flood.flood_id;
        }
        let prev_node = flood.path_trace.first();
        if prev_node.is_none() {
            return;
        }
        let (mut prev_node, _) = prev_node.unwrap();

        for (id, node_type) in flood.path_trace.iter() {
            self.graph.add_node(*id, *node_type);
            if self.graph.get_node_weight(*id) != (0.0, 0.0) {
                self.graph
                    .update_node_weight(*id, self.old_graph.get_node_weight(*id));
            }
            if *id != prev_node {
                self.graph.add_edge(*id, prev_node);
            }
            prev_node = *id;
        }
    }

    /// Increase the nack counter of the node for the pdr calculation
    pub fn node_nack(&mut self, id: NodeId) {
        let (ack, nack) = self.pdr.entry(id).or_insert((0.0, 0.0));
        *nack += 1.0;
        self.graph
            .update_node_pdr(id, *nack/ (*ack + *nack ));
    }

    pub fn nodes_ack(&mut self, header: SourceRoutingHeader) {
        if header.hops.is_empty() {
            return;
        }
        for id in header.hops.iter() {
            let (ack, nack) = self.pdr.entry(*id).or_insert((0.0, 0.0));
            *ack += 1.0;
            self.graph
            .update_node_pdr(*id, *nack/ (*ack + *nack ));
        }
    }

    /// Update the congestion of the nodes based on the SourceRoutingHeader
    ///
    /// Description:
    ///
    /// The congestion of the nodes is the normalized number of times a nodes received a packet.
    pub fn nodes_congestion(&mut self, header: SourceRoutingHeader) {
        if header.hops.is_empty() {
            return;
        }
        for id in header.hops.iter() {
            let congestion = self.congestion.entry(*id).or_insert(0.0);
            *congestion += 1.0;
        }
        let max = *self.congestion.values().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        for (key, value) in self.congestion.iter_mut() {
            self.graph
                .update_node_congestion(*key, *value / max);
        }
    }

    /// Get the best path from the start node to the end node with the a* algorithm
    ///
    /// Description:
    ///
    /// The weight of the nodes is calculated with the pdr and congestion values.
    pub fn best_path(&mut self, start: NodeId, end: NodeId) -> Option<SourceRoutingHeader> {
        match self.graph.a_star_search(start, end) {
            Ok(header) => Some(header),
            Err(e) =>{
                println!("calculating error: {}", e);
                return None;
            }
        }
    }
}
