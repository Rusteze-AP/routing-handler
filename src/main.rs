use wg_internal::packet::{FloodResponse, NodeType};

mod graph;
mod routing_handler;

fn main() {
    let mut routing = routing_handler::RoutingHandler::new();
    let flood = FloodResponse {
        flood_id: 0,
        path_trace: vec![
            (0, NodeType::Client),
            (1, NodeType::Drone),
            (2, NodeType::Drone),
            (4, NodeType::Drone),
            (8, NodeType::Drone),
            (9, NodeType::Client),
        ],
    };
    routing.update_graph(flood);
    let flood = FloodResponse {
        flood_id: 0,
        path_trace: vec![
            (0, NodeType::Client),
            (3, NodeType::Drone),
            (4, NodeType::Drone),
            (5, NodeType::Drone),
        ],
    };
    routing.update_graph(flood);
    let flood = FloodResponse {
        flood_id: 0,
        path_trace: vec![
            (0, NodeType::Client),
            (3, NodeType::Drone),
            (5, NodeType::Drone),
            (6, NodeType::Drone),
            (7, NodeType::Drone),
            (8, NodeType::Drone),
        ],
    };
    routing.update_graph(flood);

    match routing.best_path(0, 9) {
        Some(path) => {
            println!("Best path: {:?}", path);
        }
        None => {
            println!("No path found");
        }
    }
}
