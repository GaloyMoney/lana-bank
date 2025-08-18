import json
import os
import re
import psycopg2
from psycopg2.extras import RealDictCursor
import sys
import argparse

# Update output directory to be in dev/
output_dir = os.path.join('dev', 'parsed_events')

def sanitize_filename(name):
    """Remove potentially problematic characters for filenames."""
    # Remove characters that are not alphanumeric, underscore, or hyphen
    name = re.sub(r'[^\w\-]+', '_', name)
    # Avoid names starting with a dot or ending with a dot/space
    name = name.strip('._ ')
    # Limit length to avoid issues on some filesystems
    return name[:100]

def get_db_connection():
    """Get database connection using PG_CON environment variable."""
    pg_con = os.getenv('PG_CON')
    if not pg_con:
        print("Error: PG_CON environment variable not set")
        sys.exit(1)
    return psycopg2.connect(pg_con)

def main():
    # Set up argument parser
    parser = argparse.ArgumentParser(description="Export Cala events from a specified table to JSON files.")
    parser.add_argument('table_name', help="The name of the Cala events table to query (e.g., 'cala_account_events', 'cala_journal_events', 'cala_tx_template_events').")
    args = parser.parse_args()
    
    table_name = args.table_name
    # Basic validation for table name (optional, but good practice)
    if not re.match(r'^[a-zA-Z_][a-zA-Z0-9_]*$', table_name):
        print(f"Error: Invalid table name '{table_name}'. Only alphanumeric characters and underscores are allowed.")
        sys.exit(1)

    # Create the output directory if it doesn't exist
    table_output_dir = os.path.join(output_dir, sanitize_filename(table_name)) # Optional: Create sub-directory per table
    os.makedirs(table_output_dir, exist_ok=True)
    print(f"Ensured output directory exists: {table_output_dir}")

    processed_count = 0
    error_count = 0

    try:
        # Connect to the database
        print("Connecting to database...")
        conn = get_db_connection()
        
        # Create a cursor that returns results as dictionaries
        with conn.cursor(cursor_factory=RealDictCursor) as cur:
            # Construct the query dynamically and safely
            # Use psycopg2's safe parameter substitution (%s for identifiers is not standard, use AsIs or format carefully)
            # However, for table names, direct formatting is often used, ensure validation happened before.
            query = f"SELECT * FROM {table_name} ORDER BY recorded_at" # Use the table_name variable
            
            print(f"Executing query: {query}") # Show the query being run
            cur.execute(query)
            
            print(f"Fetching events from {table_name}...")
            for row in cur:
                event_id = row.get('id')
                event_json = row.get('event')  # This should already be a dict thanks to RealDictCursor

                if not event_id:
                    print(f"Skipping row: Missing 'id'")
                    error_count += 1
                    continue

                if not event_json:
                    print(f"Skipping row (ID: {event_id}): Empty 'event' data")
                    error_count += 1
                    continue

                try:
                    # Determine a descriptive part for the filename
                    event_code = event_json.get('values', {}).get('code', '')
                    event_type = event_json.get('type', 'unknown_type')
                    filename_suffix = sanitize_filename(event_code if event_code else event_type)

                    # Construct filename with a counter to ensure uniqueness
                    # Use processed_count + 1 as sequential number (e.g. 0001, 0002, etc.)
                    counter = processed_count + 1
                    filename = f"{counter:04d}_{sanitize_filename(event_id)}_{filename_suffix}.json"
                    filepath = os.path.join(table_output_dir, filename) # Save to table-specific directory

                    # Save the pretty-printed JSON to the file
                    with open(filepath, 'w', encoding='utf-8') as outfile:
                        json.dump(event_json, outfile, indent=2, ensure_ascii=False)

                    processed_count += 1
                    if processed_count % 20 == 0:  # Print progress periodically
                        print(f"Processed {processed_count} events...")

                except Exception as e:
                    print(f"An unexpected error occurred processing event (ID: {event_id}): {e}")
                    error_count += 1

        print(f"Processing complete for table '{table_name}'.")
        print(f"Successfully processed and saved: {processed_count} events.")
        print(f"Events skipped due to errors: {error_count}")
        print(f"Output files are located in the '{table_output_dir}' directory.")

    except psycopg2.Error as e:
        print(f"Database error occurred: {e}")
    except Exception as e:
        print(f"An unexpected error occurred: {e}")
    finally:
        if 'conn' in locals():
            conn.close()

if __name__ == "__main__":
    main() 