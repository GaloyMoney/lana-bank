#!/usr/bin/env bash
# Destroy a workstream (stop containers, remove worktree)
# Usage: NAME=xxx ./dev/workstream-destroy.sh

set -e

# Check for config
if [ ! -f ~/.workstreams.conf ]; then
    echo "Error: Workstreams not configured."
    exit 1
fi

source ~/.workstreams.conf

NAME="${NAME:?NAME required}"
WORKSTREAM_DIR="$WORKSTREAM_BASE/lana-bank-$NAME"

if [ ! -d "$WORKSTREAM_DIR" ]; then
    echo "Error: Workstream '$NAME' not found at $WORKSTREAM_DIR"
    exit 1
fi

echo "Destroying workstream '$NAME'..."

# Load workstream config for compose project name
if [ -f "$WORKSTREAM_DIR/.workstream.conf" ]; then
    source "$WORKSTREAM_DIR/.workstream.conf"
fi

# Stop devcontainer if running
echo "Stopping devcontainer..."
docker stop "devcontainer-$NAME" 2>/dev/null || true
docker rm "devcontainer-$NAME" 2>/dev/null || true

# Stop compose services
echo "Stopping compose services..."
export COMPOSE_PROJECT_NAME="${COMPOSE_PROJECT_NAME:-lana-bank-$NAME}"
docker compose -f "$WORKSTREAM_DIR/docker-compose.yml" \
               -f "$WORKSTREAM_DIR/docker-compose.workstream.yml" \
               down -v 2>/dev/null || true

# Close tmux window if it exists
if [ -n "$TMUX" ]; then
    tmux kill-window -t "$NAME" 2>/dev/null || true
fi

# Get branch name before removing worktree
BRANCH=$(cd "$WORKSTREAM_DIR" && git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "")

# Remove worktree
echo "Removing worktree..."
cd "$WORKSTREAM_BASE/lana-bank"
git worktree remove "$WORKSTREAM_DIR" --force 2>/dev/null || rm -rf "$WORKSTREAM_DIR"

# Optionally delete the branch
if [ -n "$BRANCH" ] && [ "$BRANCH" != "main" ] && [ "$BRANCH" != "HEAD" ]; then
    read -p "Delete branch '$BRANCH'? [y/N]: " DELETE_BRANCH
    if [[ "$DELETE_BRANCH" =~ ^[Yy]$ ]]; then
        git branch -D "$BRANCH" 2>/dev/null || echo "Branch already deleted or doesn't exist"
    fi
fi

echo ""
echo "Workstream '$NAME' destroyed."
