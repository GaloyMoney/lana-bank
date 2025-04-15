import pygraphviz as pgv
import argparse
from pathlib import Path
import sys

def load_graph(filepath: Path) -> pgv.AGraph | None:
    """Loads a DOT file into a pygraphviz AGraph object."""
    if not filepath.is_file():
        print(f"Error: Input file not found: {filepath}", file=sys.stderr)
        return None
    try:
        # strict=False allows duplicate nodes/edges if present, more robust
        graph = pgv.AGraph(filename=str(filepath), strict=False)
        return graph
    except Exception as e:
        print(f"Error parsing DOT file {filepath}: {e}", file=sys.stderr)
        return None

def diff_graphs(graph1: pgv.AGraph, graph2: pgv.AGraph, output_path: Path):
    """Compares two graphs and writes a diff DOT file."""

    nodes1 = set(graph1.nodes())
    nodes2 = set(graph2.nodes())
    edges1 = set(graph1.edges())
    edges2 = set(graph2.edges())

    common_nodes = nodes1.intersection(nodes2)
    removed_nodes = nodes1 - nodes2
    added_nodes = nodes2 - nodes1

    common_edges = edges1.intersection(edges2)
    removed_edges = edges1 - edges2
    added_edges = edges2 - edges1

    print(f"Nodes - Common: {len(common_nodes)}, Added: {len(added_nodes)}, Removed: {len(removed_nodes)}")
    print(f"Edges - Common: {len(common_edges)}, Added: {len(added_edges)}, Removed: {len(removed_edges)}")

    # Create the diff graph
    diff_graph = pgv.AGraph(directed=graph1.is_directed(), strict=False, name="DiffGraph")

    # Set graph attributes (like rankdir) from one of the originals if present
    if 'rankdir' in graph1.graph_attr:
         diff_graph.graph_attr['rankdir'] = graph1.graph_attr['rankdir']
    elif 'rankdir' in graph2.graph_attr:
         diff_graph.graph_attr['rankdir'] = graph2.graph_attr['rankdir']


    # Add nodes with styling
    # Common nodes (use attributes from graph2, black color)
    for node_name in common_nodes:
        node_g2 = graph2.get_node(node_name)
        attrs = dict(node_g2.attr)
        attrs['color'] = 'black'
        attrs['penwidth'] = 1.0 # Reset potential thickness changes
        attrs['style'] = attrs.get('style', 'filled') # Ensure style is present
        if 'fillcolor' not in attrs: # Add default fillcolor if missing
             attrs['fillcolor'] = 'lightgrey'
        diff_graph.add_node(node_name, **attrs)

    # Added nodes (use attributes from graph2, green color, bold)
    for node_name in added_nodes:
        node_g2 = graph2.get_node(node_name)
        attrs = dict(node_g2.attr)
        attrs['color'] = 'green'
        attrs['penwidth'] = 2.0
        attrs['style'] = attrs.get('style', 'filled')
        if 'fillcolor' not in attrs:
             attrs['fillcolor'] = 'lightgrey'
        diff_graph.add_node(node_name, **attrs)

    # Removed nodes (use attributes from graph1, red color, dashed)
    for node_name in removed_nodes:
        node_g1 = graph1.get_node(node_name)
        attrs = dict(node_g1.attr)
        attrs['color'] = 'red'
        attrs['style'] = 'dashed,filled' # Dashed outline, keep fill
        attrs['penwidth'] = 1.0
        if 'fillcolor' not in attrs:
             attrs['fillcolor'] = 'lightgrey'
        diff_graph.add_node(node_name, **attrs)


    # Add edges with styling
    # Common edges (black, solid)
    for edge in common_edges:
        diff_graph.add_edge(edge[0], edge[1], color='black', style='solid', penwidth=1.0)

    # Added edges (green, bold)
    for edge in added_edges:
        diff_graph.add_edge(edge[0], edge[1], color='green', style='bold', penwidth=2.0)

    # Removed edges (red, dashed)
    for edge in removed_edges:
        diff_graph.add_edge(edge[0], edge[1], color='red', style='dashed', penwidth=1.0)

    # Write the diff graph
    try:
        diff_graph.write(str(output_path))
        print(f"Successfully wrote diff graph to {output_path}")
    except Exception as e:
        print(f"Error writing diff graph to {output_path}: {e}", file=sys.stderr)

def main():
    parser = argparse.ArgumentParser(description="Compare two DOT graph files and generate a diff DOT file.")
    parser.add_argument("file1", type=Path, help="Path to the first (older) DOT file.")
    parser.add_argument("file2", type=Path, help="Path to the second (newer) DOT file.")
    parser.add_argument("output", type=Path, help="Path to write the output diff DOT file.")
    args = parser.parse_args()

    print(f"Loading graph 1: {args.file1}")
    graph1 = load_graph(args.file1)
    if not graph1:
        sys.exit(1)

    print(f"Loading graph 2: {args.file2}")
    graph2 = load_graph(args.file2)
    if not graph2:
        sys.exit(1)

    print(f"Generating diff graph: {args.output}")
    diff_graphs(graph1, graph2, args.output)

if __name__ == "__main__":
    main() 