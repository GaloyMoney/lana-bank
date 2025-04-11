// Assumes dagre, dagre-d3, d3 libraries are loaded

let allNodesData = [];
let allEdgesData = [];
let nodeDepths = {};

// Function to calculate depth of each node (using BFS) - Reusable
function calculateNodeDepths(nodes, edges) {
    const depths = {};
    const adj = {}; // Adjacency list (parent -> children)
    const inDegree = {};
    const rootNodes = [];
    const nodeMap = new Map(nodes.map(n => [n.id, n])); // For quick lookup

    nodes.forEach(node => {
        adj[node.id] = [];
        inDegree[node.id] = 0;
    });

    edges.forEach(edge => {
        // Only add edges if both source and target nodes actually exist
        if (nodeMap.has(edge.source) && nodeMap.has(edge.target)) {
            adj[edge.source].push(edge.target);
            inDegree[edge.target] = (inDegree[edge.target] || 0) + 1;
        } else {
            console.warn(`Skipping edge: Cannot find node for source "${edge.source}" or target "${edge.target}"`);
        }
    });

    nodes.forEach(node => {
        if (inDegree[node.id] === 0) {
            rootNodes.push(node.id);
            depths[node.id] = 1; // Root nodes are at depth 1
        }
    });

    const queue = [...rootNodes];
    while (queue.length > 0) {
        const u = queue.shift();
        if (adj[u]) {
            adj[u].forEach(v => {
                if (nodeMap.has(v)) { // Ensure target node exists
                    depths[v] = depths[v] || 0; // Initialize depth if not set
                    // Ensure we take the maximum depth if a node can be reached via multiple paths
                    depths[v] = Math.max(depths[v], (depths[u] || 0) + 1); // Use depths[u] || 0 for safety
                    inDegree[v]--;
                    if (inDegree[v] === 0) {
                        queue.push(v);
                    }
                }
            });
        }
    }

    // Assign depth 0 or handle nodes not reached from roots (e.g., cycles, disconnected)
    nodes.forEach(node => {
        if (depths[node.id] === undefined) {
            console.warn(`Node "${node.id}" (${node.label}) is unreachable from roots or part of a cycle. Assigning depth Infinity.`);
            depths[node.id] = Infinity; // Or handle as needed
        }
    });

    return depths;
}


// Function to render the graph based on max depth
function renderFilteredGraph(maxDepth) {
    console.log(`Rendering graph with max depth: ${maxDepth}`);

    const filteredNodes = allNodesData.filter(node => nodeDepths[node.id] <= maxDepth);
    const filteredNodeIds = new Set(filteredNodes.map(n => n.id));

    // Include edges only if both source and target are within the filtered nodes
    const filteredEdges = allEdgesData.filter(edge =>
        filteredNodeIds.has(edge.source) && filteredNodeIds.has(edge.target)
    );

    console.log(`Filtered Nodes: ${filteredNodes.length}, Filtered Edges: ${filteredEdges.length}`);


    // Create a new directed graph for rendering
    var g = new dagre.graphlib.Graph();
    g.setGraph({ rankdir: 'TB', nodesep: 50, ranksep: 70, marginx: 20, marginy: 20 }); // Added margins
    g.setDefaultEdgeLabel(() => ({}));

    // Add filtered nodes and edges
    filteredNodes.forEach(node => {
        const estimatedWidth = node.label.length * 8 + 40; // Adjust label padding
        // Add specific class based on node type for styling
        g.setNode(node.id, {
            label: node.label,
            width: Math.max(120, estimatedWidth), // Min width
            height: 40, // Slightly smaller height
            class: node.type // Add class 'account' or 'account_set'
        });
    });
    filteredEdges.forEach(edge => {
        g.setEdge(edge.source, edge.target, { arrowhead: "vee" }); // Add arrowhead style
    });

    console.log('Filtered graph construction complete. Nodes:', g.nodeCount(), 'Edges:', g.edgeCount());

    var svg = d3.select("#cala-dag-canvas"); // Select the new SVG ID
    svg.selectAll("*").remove(); // Clear previous render

    if (g.nodeCount() === 0) {
        console.warn("No nodes match the current depth filter.");
        svg.append("text")
            .attr("x", 10).attr("y", 20)
            .text(`No nodes found up to layer ${maxDepth}.`);
        return;
    }

    // --- Layout and Rendering ---
    console.log('Calculating filtered graph layout...');
    try {
        dagre.layout(g);
    } catch (layoutError) {
        console.error("Error during Dagre layout:", layoutError);
        svg.append("text")
            .attr("x", 10).attr("y", 20).attr("fill", "red")
            .text("Layout Error. Check console.");
        return;
    }
    console.log('Layout complete. Rendering filtered graph...');

    var render = new dagreD3.render();
    var svgGroup = svg.append("g");

    // Run the renderer
    render(svgGroup, g);

    // --- Zoom and Pan ---
    var initialScale = 0.85; // Slightly adjusted scale
    const svgElement = svg.node();
    const svgWidth = svgElement.clientWidth || svgElement.parentNode.clientWidth; // Get width reliably
    const svgHeight = svgElement.clientHeight || svgElement.parentNode.clientHeight; // Get height reliably
    const graphWidth = g.graph().width || 200; // Use graph bounds, provide default
    const graphHeight = g.graph().height || 200;

    var zoom = d3.zoom().on("zoom", function (event) {
        svgGroup.attr("transform", event.transform);
    });
    svg.call(zoom);

    // Calculate initial translation to center the graph
    var tx = (svgWidth - graphWidth * initialScale) / 2;
    var ty = (svgHeight - graphHeight * initialScale) / 2;
    tx = Math.max(tx, 10); // Ensure some padding
    ty = Math.max(ty, 10);

    // Apply the initial transform
    svg.call(zoom.transform, d3.zoomIdentity.translate(tx, ty).scale(initialScale));

    console.log('Filtered graph rendered.');
}

// Function to fetch data and initiate first render
async function initializeGraph() {
    try {
        console.log('Fetching Cala graph data from /api/cala-graph-data...'); // Corrected log message
        const response = await fetch('/api/cala-graph-data'); // <<< USE THE CORRECT API ENDPOINT
        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }
        const data = await response.json();
        console.log(`Received ${data.nodes?.length || 0} nodes and ${data.edges?.length || 0} edges.`);

        if (!data.nodes || !data.edges) {
            throw new Error('Invalid data format received from API.');
        }

        // Store fetched data globally
        allNodesData = data.nodes;
        allEdgesData = data.edges;

        // Calculate depths
        nodeDepths = calculateNodeDepths(allNodesData, allEdgesData);
        console.log("Node depths calculated:", nodeDepths);

        // --- Initial Render ---
        let initialDepth = Infinity;
        const depthInput = document.getElementById('depth-input');
        // depthInput.value = ''; // Keep it empty for Infinity

        // Try to find a reasonable default depth if graph isn't too deep
        const maxFiniteDepth = Math.max(0, ...Object.values(nodeDepths).filter(isFinite));
        if (maxFiniteDepth > 0 && maxFiniteDepth <= 5) { // Example: Default depth if max is 5 or less
            // initialDepth = maxFiniteDepth;
            // depthInput.value = initialDepth;
        }

        renderFilteredGraph(initialDepth);


        // --- Event Listeners ---
        document.getElementById('update-button').addEventListener('click', () => {
            const depthInput = document.getElementById('depth-input');
            let maxDepth = depthInput.value.trim() === 'Infinity' || depthInput.value.trim() === ''
                ? Infinity
                : parseInt(depthInput.value, 10);

            if (isNaN(maxDepth) || maxDepth < 1) {
                if (depthInput.value.trim() !== '' && depthInput.value.trim() !== 'Infinity') {
                    console.warn(`Invalid depth input "${depthInput.value}". Defaulting to Infinity.`);
                }
                maxDepth = Infinity;
                depthInput.value = ''; // Clear invalid input
            }
            renderFilteredGraph(maxDepth);
        });

        document.getElementById('depth-input').addEventListener('keypress', (event) => {
            if (event.key === 'Enter') {
                event.preventDefault();
                document.getElementById('update-button').click();
            }
        });

    } catch (error) {
        console.error('Failed to fetch or initialize Cala graph:', error);
        d3.select("#cala-dag-canvas") // Use the new ID
            .attr("x", 10).attr("y", 20).attr("fill", "red")
            .text("Error initializing graph. Check console.");
    }
}

// Run the initialization function when the script loads
initializeGraph(); 