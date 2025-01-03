use crate::graph::Graph;
use logger::{LogLevel, Logger};
use std::{collections::HashMap, u64};
use wg_internal::{
    network::{NodeId, SourceRoutingHeader},
    packet::FloodResponse,
};

pub struct RoutingHandler {
    graph: Graph,
    old_graph: Graph,
    current_flood_id: u64,
    pdr: HashMap<NodeId, (u64, u64)>,
    congestion: HashMap<NodeId, u64>,
    logger: Logger,
}

impl RoutingHandler {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            old_graph: Graph::new(),
            current_flood_id: u64::MAX,
            pdr: HashMap::new(),
            congestion: HashMap::new(),
            logger: Logger::new(LogLevel::All as u8, false, "RoutingHandler".to_string()),
        }
    }

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
            if self.graph.get_node_weight(*id) != (0.0,0.0){
                self.graph.update_node_weight(*id, self.old_graph.get_node_weight(*id));
            }
            if *id != prev_node {
                self.graph.add_edge(*id, prev_node);
            }
            prev_node = *id;
        }
        self.logger
            .log_debug(format!("GRAPH: {:?}", self.graph).as_str()); 
    }

    pub fn node_ack(&mut self, id: NodeId) {
        let (ack, nack) = self.pdr.entry(id).or_insert((0, 0));
        *ack += 1;
        self.graph.update_node_pdr(id, (*ack / *nack) as f32);
    }

    pub fn node_nack(&mut self, id: NodeId) {
        let (ack, nack) = self.pdr.entry(id).or_insert((0, 0));
        *nack += 1;
        self.graph.update_node_pdr(id, (*ack / *nack) as f32);
    }

    pub fn nodes_congestion(&mut self, header: SourceRoutingHeader) {
        for id in header.hops.iter() {
            let congestion = self.congestion.entry(*id).or_insert(0);
            *congestion += 1;
        }
        let max = *self.congestion.values().max().unwrap();
        for (key, value) in self.congestion.iter_mut() {
            *value = *value / max;
            self.graph.update_node_congestion(*key, *value as f32);
        }
    }

    pub fn best_path(&mut self, start: NodeId, end: NodeId) -> Option<SourceRoutingHeader> {
        match self.graph.a_star_search(start, end) {
            Ok(header) => {
                self.logger.log_debug(
                    format!("Best path from {} to {} is {:?}", start, end, header).as_str(),
                );
                Some(header)
            }
            Err(e) => {
                self.logger.log_error(e.as_str());
                None
            }
        }
    }

    // //TODO: remove these functions
    // // test only
    // pub fn update_congestion(&mut self, id: NodeId, congestion: f32) {
    //     self.graph.update_congestion(id, congestion);
    // }
    // // test only
    // pub fn update_pdr(&mut self, id: NodeId, pdr: f32) {
    //     self.graph.update_pdr(id, pdr);
    // }
}
