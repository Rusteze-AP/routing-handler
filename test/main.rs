use wg_internal::packet::{FloodResponse, NodeType};

mod graph;
mod routing_handler;

fn main() {
    let mut routing = routing_handler::RoutinHandler::new();
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
    routing.update_congestion(1, 2.0);
    routing.update_congestion(2, 20.0);
    routing.update_congestion(3, 10.0);
    routing.update_congestion(4, 50.0);
    routing.update_congestion(5, 5.0);
    routing.update_congestion(6, 2.0);
    routing.update_congestion(7, 5.0);
    routing.update_congestion(8, 13.0);

    routing.best_path(0, 9);
}
