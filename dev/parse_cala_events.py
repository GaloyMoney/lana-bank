import json
import os
import re
import psycopg2
from psycopg2.extras import RealDictCursor
import sys

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
    # Create the output directory if it doesn't exist
    os.makedirs(output_dir, exist_ok=True)
    print(f"Ensured output directory exists: {output_dir}")

    processed_count = 0
    error_count = 0

    try:
        # Connect to the database
        print("Connecting to database...")
        conn = get_db_connection()
        
        # Create a cursor that returns results as dictionaries
        with conn.cursor(cursor_factory=RealDictCursor) as cur:
            # Execute the query
            cur.execute("SELECT * FROM cala_tx_template_events ORDER BY recorded_at")
            
            print("Fetching events from database...")
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

                    # Construct filename
                    filename = f"{sanitize_filename(event_id)}_{filename_suffix}.json"
                    filepath = os.path.join(output_dir, filename)

                    # Save the pretty-printed JSON to the file
                    with open(filepath, 'w', encoding='utf-8') as outfile:
                        json.dump(event_json, outfile, indent=2, ensure_ascii=False)

                    processed_count += 1
                    if processed_count % 20 == 0:  # Print progress periodically
                        print(f"Processed {processed_count} events...")

                except Exception as e:
                    print(f"An unexpected error occurred processing event (ID: {event_id}): {e}")
                    error_count += 1

        print(f"\nProcessing complete.")
        print(f"Successfully processed and saved: {processed_count} events.")
        print(f"Events skipped due to errors: {error_count}")
        print(f"Output files are located in the '{output_dir}' directory.")

    except psycopg2.Error as e:
        print(f"Database error occurred: {e}")
    except Exception as e:
        print(f"An unexpected error occurred: {e}")
    finally:
        if 'conn' in locals():
            conn.close()

if __name__ == "__main__":
    main() 