import os
import sys
import psycopg2
import json
from datetime import datetime
from pathlib import Path

# --- Configuration ---
OUTPUT_DIR = Path("output")

# --- Database Connection ---
connection_string = os.environ.get("PG_CON")
if not connection_string:
    print("Error: PG_CON environment variable is not set.", file=sys.stderr)
    print("Please set PG_CON to your PostgreSQL connection string.", file=sys.stderr)
    sys.exit(1)

# --- DOT Generation Functions ---

def escape_dot(s):
    """Escapes strings for DOT labels."""
    if not isinstance(s, str):
        return s
    # Corrected escaping for double quotes and newlines within a Python string literal
    return s.replace('"', '\\"').replace('\n', '\\n')

def generate_lana_dot(conn, output_filepath: Path):
    """Generates the Lana chart of accounts DOT graph."""
    print(f"Generating Lana DOT file at {output_filepath}...")
    try:
        with conn.cursor() as cur:
            query = "SELECT id, event, event_type FROM core_chart_events ORDER BY id;"
            cur.execute(query)
            rows = cur.fetchall()
            print(f"Fetched {len(rows)} rows for Lana DAG.")

            dot_lines = ["digraph Lana {", "  rankdir=LR;", "  node [shape=box, style=filled, fillcolor=lightgrey];", ""]
            node_map = {} # Map node ID -> { 'id': str, 'code': str | None, 'event': dict }
            edges_to_add = [] # List of { 'source': str, 'target': str }

            # First pass: Define nodes
            for row_id, event_json, event_type in rows:
                node_id = None
                node_label = None
                code_string = None

                try:
                    event_data = json.loads(event_json) if isinstance(event_json, str) else event_json

                    if event_type == 'initialized':
                        node_id = event_data.get('ledger_account_set_id') or f"init_node_{row_id}"
                        # Handle name being a string or a dict
                        name_field = event_data.get('name')
                        if isinstance(name_field, dict):
                            name = name_field.get('name', 'Chart of Accounts') # Get name from inner dict
                        elif isinstance(name_field, str):
                            name = name_field # Use the string directly
                        else:
                            name = 'Chart of Accounts' # Default if name is missing or unexpected type

                        node_label = f"{escape_dot(name)}\n(Type: {escape_dot(event_type)})"
                    elif event_type == 'node_added' and 'spec' in event_data:
                        node_id = event_data.get('ledger_account_set_id')
                        if not node_id:
                            print(f"Warning: Missing ledger_account_set_id for node_added event, row id: {row_id}. Skipping node.", file=sys.stderr)
                            continue
                        name = event_data['spec'].get('name', {}).get('name', 'Unnamed Node')
                        node_label = f"{escape_dot(name)}\n(Type: {escape_dot(event_type)})"
                        if 'code' in event_data['spec'] and 'sections' in event_data['spec']['code']:
                             code_sections = event_data['spec']['code']['sections']
                             if isinstance(code_sections, list):
                                 code_string = ''.join([s.get('code', '') for s in code_sections])
                    else:
                        node_id = f"event_{row_id}"
                        node_label = f"{escape_dot(json.dumps(event_data))}\n(Type: {escape_dot(event_type)})"

                    if node_id:
                        dot_lines.append(f'  "{node_id}" [label="{node_label}"];')
                        node_map[node_id] = {'id': node_id, 'code': code_string, 'event': event_data}

                except json.JSONDecodeError as e:
                    print(f"Error parsing event JSON for row id: {row_id}, event: {event_json}: {e}", file=sys.stderr)
                    node_id = f"error_node_{row_id}"
                    node_label = f"Error parsing event\n(Row ID: {row_id})"
                    dot_lines.append(f'  "{node_id}" [label="{node_label}", fillcolor=red];')
                except Exception as e:
                    print(f"Unexpected error processing row id: {row_id}: {e}", file=sys.stderr)
                    node_id = f"error_node_{row_id}"
                    node_label = f"Error processing event\n(Row ID: {row_id})"
                    dot_lines.append(f'  "{node_id}" [label="{node_label}", fillcolor=orange];')


            dot_lines.append("\n  // Edges")

            # Second pass: Determine edges
            for child_id, child_node in node_map.items():
                if child_node['event'].get('type') != 'node_added':
                     continue

                spec = child_node['event'].get('spec')
                if not spec:
                    continue

                parent_info = spec.get('parent')
                if not parent_info or 'sections' not in parent_info:
                    continue

                parent_sections = parent_info['sections']
                if not isinstance(parent_sections, list):
                    continue

                parent_code_string = ''.join([s.get('code', '') for s in parent_sections])
                parent_node_id = None

                # Find parent by matching code string
                for potential_parent_id, potential_parent_node in node_map.items():
                    if potential_parent_node['code'] is not None and potential_parent_node['code'] == parent_code_string:
                        parent_node_id = potential_parent_id
                        break

                if parent_node_id:
                    dot_lines.append(f'  "{parent_node_id}" -> "{child_id}";')
                elif parent_code_string: # Don't warn if parent spec was empty/null
                    child_name = spec.get('name', {}).get('name', 'Unnamed Node')
                    print(f"Warning: Could not find parent node with code [{parent_code_string}] for child node {child_id} ({child_name})", file=sys.stderr)


            dot_lines.append("}")

            output_filepath.write_text("\n".join(dot_lines))
            print(f"Successfully wrote Lana graph to {output_filepath}")

    except (Exception, psycopg2.DatabaseError) as error:
        print(f"Error generating Lana DOT to {output_filepath}: {error}", file=sys.stderr)
        raise

def generate_cala_dot(conn, output_filepath: Path):
    """Generates the Cala accounts and sets DOT graph."""
    print(f"Generating Cala DOT file at {output_filepath}")
    try:
        with conn.cursor() as cur:
            # Fetch Accounts
            accounts_query = """
                SELECT id::text, COALESCE(name, 'Unnamed Account') as name, COALESCE(code, '') as code, external_id::text
                FROM cala_accounts;
            """
            cur.execute(accounts_query)
            accounts = cur.fetchall()

            # Fetch Account Sets
            sets_query = """
                SELECT id::text, COALESCE(name, 'Unnamed Set') as name, external_id::text
                FROM cala_account_sets;
            """
            cur.execute(sets_query)
            account_sets = cur.fetchall()

            # Fetch Account -> Set Memberships
            member_acc_query = """
                SELECT account_set_id::text as source, member_account_id::text as target
                FROM cala_account_set_member_accounts WHERE transitive = false;
            """
            cur.execute(member_acc_query)
            acc_memberships = cur.fetchall()

            # Fetch Set -> Set Memberships
            member_set_query = """
                SELECT account_set_id::text as source, member_account_set_id::text as target
                FROM cala_account_set_member_account_sets;
            """
            cur.execute(member_set_query)
            set_memberships = cur.fetchall()

        dot_lines = ["digraph Cala {", "  rankdir=LR;", "  node [shape=record, style=filled];", ""]

        # Add account nodes
        dot_lines.append("  // Account Nodes")
        for acc_id, name, code, _ in accounts:
            label_parts = [escape_dot(name)]
            if code:
                label_parts.append(f"Code: {escape_dot(code)}")
            label = f"{{{ ' | '.join(label_parts) }}}"
            dot_lines.append(f'  "{escape_dot(acc_id)}" [label="{label}", fillcolor=lightblue];')
        dot_lines.append("")

        # Add set nodes
        dot_lines.append("  // Set Nodes")
        for set_id, name, _ in account_sets:
            label_parts = [escape_dot(name)]
            label = f"{{{ ' | '.join(label_parts) }}}"
            dot_lines.append(f'  "{escape_dot(set_id)}" [label="{label}", shape=folder, fillcolor=lightcoral];')
        dot_lines.append("")

        node_ids = set(acc[0] for acc in accounts) | set(s[0] for s in account_sets)
        all_edges = acc_memberships + set_memberships

        # Filter out edges pointing to/from non-existent nodes
        valid_edges = [(src, tgt) for src, tgt in all_edges if src in node_ids and tgt in node_ids]
        invalid_edge_count = len(all_edges) - len(valid_edges)
        if invalid_edge_count > 0:
            print(f"Warning: Filtered out {invalid_edge_count} Cala edges with missing nodes.", file=sys.stderr)

        print(f"Fetched {len(node_ids)} nodes and {len(valid_edges)} edges for Cala DAG.")

        # Add edges
        dot_lines.append("  // Edges (Memberships)")
        for source, target in valid_edges:
            dot_lines.append(f'  "{escape_dot(source)}" -> "{escape_dot(target)}";')

        dot_lines.append("}")

        output_filepath.write_text("\n".join(dot_lines))
        print(f"Successfully wrote Cala graph to {output_filepath}")

    except (Exception, psycopg2.DatabaseError) as error:
        print(f"Error generating Cala DOT to {output_filepath}: {error}", file=sys.stderr)
        raise


# --- Main Execution ---
if __name__ == "__main__":
    conn = None
    try:
        # Generate timestamp
        now = datetime.now()
        timestamp = now.strftime("%Y%m%d%H%M%S")

        # Ensure output directory exists
        OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
        print(f"Ensured output directory exists: {OUTPUT_DIR}")

        # Construct timestamped filenames
        lana_dot_file = OUTPUT_DIR / f"lana_{timestamp}.dot"
        cala_dot_file = OUTPUT_DIR / f"cala_{timestamp}.dot"

        print("Connecting to database...")
        conn = psycopg2.connect(connection_string)
        print("Database connection successful.")

        generate_lana_dot(conn, lana_dot_file)
        generate_cala_dot(conn, cala_dot_file)

        print("DOT file generation complete.")

    except (Exception, psycopg2.DatabaseError) as error:
        print(f"An error occurred during DOT generation: {error}", file=sys.stderr)
        sys.exit(1) # Indicate failure
    finally:
        if conn:
            print("Closing database connection.")
            conn.close()
            print("Database connection closed.") 