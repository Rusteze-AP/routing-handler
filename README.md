# routing-handler
This repository handles the routing of the network. Using the Flood response, it constructs a graph with all the connections between nodes.

To find the best path, it uses the A* algorithm on the weighted graph.

The weights of the graph are calculated based on node congestion and the estimated PDR. This way, the best path consists of nodes with a lower PDR and nodes that are less utilized in the network.

When a new Flood response with a different session ID is used to update the graph, a new graph is created while retaining the old weights. This ensures that if some nodes crash, they will not be present in the new graph, but the old nodes will still retain the past history of PDR and congestion.