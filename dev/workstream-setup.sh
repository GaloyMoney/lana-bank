#!/usr/bin/env bash
# One-time setup for workstreams
# Usage: make setup-workstreams

set -e

echo "Workstreams Setup"
echo "================="
echo ""

# Check if already configured
if [ -f ~/.workstreams.conf ]; then
    source ~/.workstreams.conf
    echo "Existing configuration found: $WORKSTREAM_BASE"
    read -p "Reconfigure? [y/N]: " RECONFIGURE
    if [[ ! "$RECONFIGURE" =~ ^[Yy]$ ]]; then
        echo "Keeping existing configuration."
        exit 0
    fi
fi

read -p "Enter base directory for workstreams [default: ~/workstreams/lana-bank]: " WORKSTREAM_BASE
WORKSTREAM_BASE="${WORKSTREAM_BASE:-$HOME/workstreams/lana-bank}"

# Expand ~
WORKSTREAM_BASE="${WORKSTREAM_BASE/#\~/$HOME}"

# Create directory
mkdir -p "$WORKSTREAM_BASE"

# Store config
cat > ~/.workstreams.conf << EOF
WORKSTREAM_BASE="$WORKSTREAM_BASE"
EOF

# Get current repo location
CURRENT_REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Create symlink or copy for main repo if not already there
MAIN_REPO="$WORKSTREAM_BASE/lana-bank"
if [ ! -d "$MAIN_REPO" ]; then
    if [ "$CURRENT_REPO" != "$MAIN_REPO" ]; then
        echo ""
        echo "Creating symlink to main repo..."
        ln -s "$CURRENT_REPO" "$MAIN_REPO"
        echo "Linked: $MAIN_REPO -> $CURRENT_REPO"
    fi
fi

echo ""
echo "Setup complete!"
echo "Workstreams will be created in: $WORKSTREAM_BASE"
echo ""
echo "Next steps:"
echo "  make workstream <name>           - Create a new workstream"
echo "  make workstream <name> <branch>  - Create workstream with custom branch"
echo "  make list-workstreams            - List all workstreams"
