use std::collections::{BinaryHeap, HashMap, HashSet};
use wg_internal::{
    network::{NodeId, SourceRoutingHeader},
    packet::NodeType,
};

const PDR_WEIGHT: f32 = 1.0;
const CONGESTION_WEIGHT: f32 = 1.0;
const WEIGHT_FACTOR: f32 = 1.0;

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub node_type: NodeType,
    pub congestion: f32,
    pub pdr: f32,
    pub predictions: HashMap<NodeId, f32>,
}

impl Node {
    pub fn new(node_type: NodeType) -> Self {
        Self {
            node_type,
            congestion: 0.0,
            pdr: 0.0,
            predictions: HashMap::new(),
        }
    }

    pub fn update_congestion(&mut self, congestion: f32) {
        self.congestion = congestion;
    }

    pub fn update_pdr(&mut self, pdr: f32) {
        self.pdr = pdr;
    }

    pub fn get_weight(&self) -> f32 {
        WEIGHT_FACTOR
            / ((1.0 - self.pdr) * PDR_WEIGHT + (1.0 - self.congestion) * CONGESTION_WEIGHT)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct AStarNode {
    id: NodeId,
    weight: f32,
}

impl Eq for AStarNode {}

impl Ord for AStarNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.weight
            .partial_cmp(&other.weight)
            .unwrap_or(std::cmp::Ordering::Equal)
            .reverse()
    }
}

impl PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone)]
pub struct Graph {
    nodes: HashMap<NodeId, Node>,
    graph: HashMap<NodeId, HashSet<NodeId>>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            graph: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.graph.clear();
        
    }

    pub fn add_node(&mut self, id: NodeId, node_type: NodeType) {
        if self.nodes.contains_key(&id) {
            return;
        }
        self.nodes.insert(id, Node::new(node_type));
    }

    pub fn add_edge(&mut self, n1: NodeId, n2: NodeId) {
        self.graph.entry(n1).or_insert_with(HashSet::new).insert(n2);
        self.graph.entry(n2).or_insert_with(HashSet::new).insert(n1);
    }

    pub fn get_node_weight(&self, id: NodeId) -> (f32,f32) {
        if let Some(node) = self.nodes.get(&id) {
            return (node.pdr, node.congestion);
        }
        (0.0,0.0)
    }

    pub fn update_node_weight(&mut self, id: NodeId, weight: (f32,f32)) {
        if let Some(node) = self.nodes.get_mut(&id) {
            node.pdr = weight.0;
            node.congestion = weight.1;
        }
    }

    pub fn update_node_congestion(&mut self, id: NodeId, congestion: f32) {
        if let Some(node) = self.nodes.get_mut(&id) {
            node.update_congestion(congestion);
        }
    }

    pub fn update_node_pdr(&mut self, id: NodeId, pdr: f32) {
        if let Some(node) = self.nodes.get_mut(&id) {
            node.update_pdr(pdr);
        }
    }

    pub fn a_star_search(
        &mut self,
        start: NodeId,
        end: NodeId,
    ) -> Result<SourceRoutingHeader, String> {
        match self.nodes.get(&start) {
            Some(node) => {
                if node.node_type == NodeType::Drone {
                    return Err("Start node is a drone".to_string());
                }
            }
            None => return Err("Start node does not exist".to_string()),
        }
        match self.nodes.get(&end) {
            Some(node) => {
                if node.node_type == NodeType::Drone {
                    return Err("End node is a drone".to_string());
                }
            }
            None => return Err("End node does not exist".to_string()),
        }
        if start == end {
            return Err("Start and end nodes are the same".to_string());
        }

        let mut open_set: BinaryHeap<AStarNode> = BinaryHeap::new();
        let mut came_from: HashMap<NodeId, NodeId> = HashMap::new();
        let mut g_score: HashMap<NodeId, f32> = HashMap::new();
        let mut f_score: HashMap<NodeId, f32> = HashMap::new();

        g_score.insert(start, 0.0);
        f_score.insert(start, self.a_star_heuristics(start, end));
        open_set.push(AStarNode {
            id: start,
            weight: f_score[&start],
        });

        while let Some(current) = open_set.pop() {
            if current.id == end {
                return Ok(self.reconstruct_path(came_from, current.id));
            }

            for neighbor in self.graph[&current.id].iter() {
                if self.nodes[neighbor].node_type != NodeType::Drone && *neighbor != end {
                    continue;
                }
                let tentative_g_score = g_score[&current.id] + self.nodes[neighbor].get_weight();
                if tentative_g_score < *g_score.get(neighbor).unwrap_or(&f32::INFINITY) {
                    came_from.insert(*neighbor, current.id);
                    g_score.insert(*neighbor, tentative_g_score);
                    f_score.insert(
                        *neighbor,
                        tentative_g_score + self.a_star_heuristics(*neighbor, end),
                    );
                    open_set.push(AStarNode {
                        id: *neighbor,
                        weight: f_score[neighbor],
                    });
                }
            }
        }

        Err("No path found".to_string())
    }

    fn a_star_heuristics(&self, current: NodeId, end: NodeId) -> f32 {
        if let Some(value) = self.nodes.get(&current).unwrap().predictions.get(&end) {
            return *value;
        }
        0.0
    }

    fn reconstruct_path(
        &mut self,
        came_from: HashMap<NodeId, NodeId>,
        end: NodeId,
    ) -> SourceRoutingHeader {
        let mut total_path = SourceRoutingHeader::initialize(vec![]);
        total_path.append_hop(end);
        let mut weight = self.nodes.get(&end).unwrap().get_weight();
        self.nodes
            .get_mut(&end)
            .unwrap()
            .predictions
            .insert(end, weight);

        let mut current = end;
        while came_from.contains_key(&current) {
            current = came_from[&current];
            total_path.append_hop(current);

            weight += self.nodes.get(&current).unwrap().get_weight();
            self.nodes
                .get_mut(&current)
                .unwrap()
                .predictions
                .insert(end, weight);
        }
        total_path.reverse();
        total_path.hop_index = 1;
        total_path
    }
}
