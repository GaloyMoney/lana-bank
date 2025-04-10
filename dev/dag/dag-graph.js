// Assumes dagre library is loaded, e.g., via <script> tag or require('dagre')

// Global variables to store processed data
let allNodesData = [];
let allEdgesData = [];
let nodeDepths = {};

// Helper function to generate a node ID from sections
function getNodeId(sections) {
    if (!sections || !Array.isArray(sections)) return null;
    return sections.map(s => s.code).join('-');
}

// Function to calculate depth of each node (using BFS)
function calculateNodeDepths(nodes, edges) {
    const depths = {};
    const adj = {}; // Adjacency list (parent -> children)
    const inDegree = {};
    const rootNodes = [];

    nodes.forEach(node => {
        adj[node.id] = [];
        inDegree[node.id] = 0;
    });

    edges.forEach(edge => {
        if (adj[edge.source]) { // Check if source exists in nodes list
            adj[edge.source].push(edge.target);
            inDegree[edge.target] = (inDegree[edge.target] || 0) + 1;
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
                if (inDegree[v] !== undefined) { // Check if target exists
                    depths[v] = (depths[v] || 0);
                    // Ensure we take the maximum depth if a node can be reached via multiple paths
                    depths[v] = Math.max(depths[v], depths[u] + 1);
                    inDegree[v]--;
                    if (inDegree[v] === 0) {
                        queue.push(v);
                    }
                }
            });
        }
    }
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

    // Create a new directed graph for rendering
    var g = new dagre.graphlib.Graph();
    g.setGraph({ rankdir: 'TB', nodesep: 50, ranksep: 50 });
    g.setDefaultEdgeLabel(() => ({}));

    // Add filtered nodes and edges
    filteredNodes.forEach(node => {
        g.setNode(node.id, node.config);
    });
    filteredEdges.forEach(edge => {
        g.setEdge(edge.source, edge.target);
    });

    console.log('Filtered graph construction complete. Nodes:', g.nodeCount(), 'Edges:', g.edgeCount());

    if (g.nodeCount() === 0) {
        console.warn("No nodes match the current depth filter.");
        // Clear previous graph and display a message
        var svg = d3.select("#dag-canvas");
        svg.selectAll("*").remove();
        svg.append("text")
            .attr("x", 10).attr("y", 20)
            .text(`No nodes found up to layer ${maxDepth}.`);
        return;
    }

    // --- Layout and Rendering (Similar to before) ---
    console.log('Calculating filtered graph layout...');
    try {
        dagre.layout(g);
    } catch (layoutError) {
        console.error("Error during Dagre layout:", layoutError);
        // Handle layout errors, maybe display a message
        var svg = d3.select("#dag-canvas");
        svg.selectAll("*").remove();
        svg.append("text")
            .attr("x", 10).attr("y", 20).attr("fill", "red")
            .text("Layout Error. Check console.");
        return;
    }

    console.log('Layout complete. Rendering filtered graph...');

    var render = new dagreD3.render();
    var svg = d3.select("#dag-canvas");
    svg.selectAll("*").remove();
    var svgGroup = svg.append("g");

    render(svgGroup, g);

    var initialScale = 0.75;
    const svgElement = svg.node();
    const svgWidth = svgElement.clientWidth;
    const svgHeight = svgElement.clientHeight;
    const graphWidth = g.graph().width * initialScale;
    const graphHeight = g.graph().height * initialScale;

    var zoom = d3.zoom().on("zoom", function (event) {
        svgGroup.attr("transform", event.transform);
    });
    svg.call(zoom);

    var tx = (svgWidth - graphWidth) / 2;
    var ty = (svgHeight - graphHeight) / 2;
    tx = Math.max(tx, 5);
    ty = Math.max(ty, 5);

    svg.call(zoom.transform, d3.zoomIdentity.translate(tx, ty).scale(initialScale));

    console.log('Filtered graph rendered.');
}

// Function to fetch data and initiate first render
async function initializeGraph() {
    try {
        console.log('Fetching graph data from /api/graph-data...');
        const response = await fetch('/api/graph-data');
        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }
        const events = await response.json();
        console.log(`Received ${events.length} events from API.`);

        // Clear previous data
        allNodesData = [];
        allEdgesData = [];
        nodeDepths = {};

        // Temporary map to build graph structure for depth calculation
        const tempNodes = new Map();
        const tempEdges = [];

        // Process events to populate global data arrays
        events.forEach((item, index) => {
            let eventData = item.event;
            if (typeof eventData !== 'object' || eventData === null) {
                console.warn(`  Skipping event ${index + 1}: item.event is not a valid object`, item.event);
                return;
            }

            if (eventData.type === 'node_added' && eventData.spec && eventData.spec.code && eventData.spec.code.sections) {
                const spec = eventData.spec;
                const nodeId = getNodeId(spec.code.sections);
                const nodeLabel = spec.name?.name || nodeId;
                const parentSections = spec.parent?.sections;
                const parentId = getNodeId(parentSections);

                if (nodeId && !tempNodes.has(nodeId)) {
                    const estimatedWidth = nodeLabel.length * 8 + 20;
                    const nodeConfig = { label: nodeLabel, width: Math.max(100, estimatedWidth), height: 50 };
                    allNodesData.push({ id: nodeId, config: nodeConfig });
                    tempNodes.set(nodeId, true);
                }

                if (parentId && nodeId) {
                    // Add edge data even if parent node appears later in the stream
                    allEdgesData.push({ source: parentId, target: nodeId });
                    tempEdges.push({ source: parentId, target: nodeId });
                }
            }
        });

        // Calculate depths using the processed data
        nodeDepths = calculateNodeDepths(allNodesData, allEdgesData);
        console.log("Node depths calculated:", nodeDepths);

        // Initial render (show all layers by default)
        const initialDepth = Infinity;
        document.getElementById('depth-input').value = initialDepth;
        renderFilteredGraph(initialDepth);

        // Add event listener for the update button
        document.getElementById('update-button').addEventListener('click', () => {
            const depthInput = document.getElementById('depth-input');
            let maxDepth = parseInt(depthInput.value, 10);
            if (isNaN(maxDepth) || maxDepth < 1) {
                maxDepth = Infinity; // Default to showing all if input is invalid
                depthInput.value = 'Infinity'; // Reset input visually
            }
            renderFilteredGraph(maxDepth);
        });

        // Add event listener for Enter key in the input field
        document.getElementById('depth-input').addEventListener('keypress', (event) => {
            if (event.key === 'Enter') {
                event.preventDefault(); // Prevent potential form submission
                document.getElementById('update-button').click(); // Trigger button click
            }
        });


    } catch (error) {
        console.error('Failed to fetch or initialize graph:', error);
        d3.select("#dag-canvas").append("text")
            .attr("x", 10).attr("y", 20).attr("fill", "red")
            .text("Error initializing graph. Check console.");
    }
}

// Run the initialization function when the script loads
initializeGraph(); 