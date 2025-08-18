#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

# Ensure we are in the script's directory (dev/dag)
# This makes sure paths like 'output/' are relative to this script
cd "$(dirname "$0")"

# Check for Python 3 (should be provided by Nix env)
if ! command -v python3 &> /dev/null
then
    echo "Error: python3 could not be found. Is it included in your Nix environment?" >&2
    exit 1
fi

# Dependencies are now handled by Nix, removed pip check and install

echo "Generating timestamped DOT files via python3 generate_dots.py..."
python3 generate_dots.py # Use the Python script

echo "Finding the latest DOT files..."
# Use find for potentially more robust handling of filenames and error cases
LATEST_LANA_DOT=$(find output -maxdepth 1 -name 'lana_*.dot' -printf '%T@ %p\n' | sort -nr | head -n 1 | cut -d' ' -f2-)
LATEST_CALA_DOT=$(find output -maxdepth 1 -name 'cala_*.dot' -printf '%T@ %p\n' | sort -nr | head -n 1 | cut -d' ' -f2-)

# Check if files were found
if [ -z "$LATEST_LANA_DOT" ]; then
  echo "Error: Could not find any lana_*.dot files in output/. Did 'python3 generate_dots.py' run correctly and create the file?" >&2
  exit 1
fi
if [ -z "$LATEST_CALA_DOT" ]; then
  echo "Error: Could not find any cala_*.dot files in output/. Did 'python3 generate_dots.py' run correctly and create the file?" >&2
  exit 1
fi

echo "Latest Lana DOT: $LATEST_LANA_DOT"
echo "Latest Cala DOT: $LATEST_CALA_DOT"

# Construct PNG filenames
LATEST_LANA_PNG="${LATEST_LANA_DOT%.dot}.png"
LATEST_CALA_PNG="${LATEST_CALA_DOT%.dot}.png"

# Generate PNGs using Graphviz dot
echo "Generating PNG: $LATEST_LANA_PNG"
# Check if dot command exists (should be provided by Nix env)
if ! command -v dot &> /dev/null
then
    echo "Error: 'dot' command (from Graphviz) not found. Is Graphviz included in your Nix environment?" >&2
    exit 1
fi
dot -Tpng "$LATEST_LANA_DOT" -o "$LATEST_LANA_PNG" -s 72

echo "Generating PNG: $LATEST_CALA_PNG"
dot -Tpng "$LATEST_CALA_DOT" -o "$LATEST_CALA_PNG" -s 72

echo "Successfully generated PNG files:"
echo "  $LATEST_LANA_PNG"
echo "  $LATEST_CALA_PNG"