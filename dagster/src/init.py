import os
import sys


def main():
    mode = os.environ.get("DAGSTER_INIT_MODE", "")

    if mode == "dry-run":
        print("DAGSTER_INIT_MODE=dry-run, skipping BigQuery UDF creation")
        return

    # Do the init here
    print("Init completed")


if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        print(f"Init failed: {e}", file=sys.stderr)
        sys.exit(1)
