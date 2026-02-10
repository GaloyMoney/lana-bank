#!/usr/bin/env bash
# Create a new workstream with git worktree and devcontainer setup
# Usage: NAME=xxx [BRANCH=branch] ./dev/workstream-create.sh

set -e

# Check for config
if [ ! -f ~/.workstreams.conf ]; then
    echo "Error: Workstreams not configured. Run 'make setup-workstreams' first."
    exit 1
fi

source ~/.workstreams.conf

NAME="${NAME:?NAME required}"
BRANCH="${BRANCH:-feat/$NAME}"
MAIN_REPO="$WORKSTREAM_BASE/lana-bank"
WORKSTREAM_DIR="$WORKSTREAM_BASE/lana-bank-$NAME"

# Calculate port offset
# Find highest existing offset and add 10000
get_next_offset() {
    local max_offset=0
    shopt -s nullglob
    for conf in "$WORKSTREAM_BASE"/lana-bank-*/.workstream.conf; do
        if [ -f "$conf" ]; then
            local offset
            offset=$(grep WORKSTREAM_PORT_OFFSET "$conf" | cut -d= -f2)
            if [ "$offset" -gt "$max_offset" ]; then
                max_offset=$offset
            fi
        fi
    done
    shopt -u nullglob
    echo $((max_offset + 10000))
}

# Check if workstream already exists
if [ -d "$WORKSTREAM_DIR" ]; then
    echo "Workstream '$NAME' already exists at $WORKSTREAM_DIR"

    if [ -n "$TMUX" ]; then
        # Already in tmux, switch to or create window
        if tmux list-windows -F '#{window_name}' | grep -q "^${NAME}$"; then
            tmux select-window -t "$NAME"
        else
            tmux new-window -n "$NAME" -c "$WORKSTREAM_DIR"
        fi
        echo "Switched to tmux window '$NAME'"
    else
        # Not in tmux, create/attach session
        if tmux has-session -t "lana-$NAME" 2>/dev/null; then
            exec tmux attach -t "lana-$NAME"
        else
            tmux new-session -d -s "lana-$NAME" -n "$NAME" -c "$WORKSTREAM_DIR"
            exec tmux attach -t "lana-$NAME"
        fi
    fi
    exit 0
fi

OFFSET=$(get_next_offset)

echo "Creating workstream '$NAME'..."
echo "  Directory: $WORKSTREAM_DIR"
echo "  Branch: $BRANCH"
echo "  Port offset: +$OFFSET"

# Ensure main repo exists
if [ ! -d "$MAIN_REPO" ]; then
    echo "Error: Main repo not found at $MAIN_REPO"
    echo "Run 'make setup-workstreams' first."
    exit 1
fi

cd "$MAIN_REPO"
git fetch origin

# Create worktree
if git show-ref --verify --quiet "refs/heads/$BRANCH" 2>/dev/null; then
    # Local branch exists
    echo "Using existing local branch: $BRANCH"
    git worktree add "$WORKSTREAM_DIR" "$BRANCH"
elif git show-ref --verify --quiet "refs/remotes/origin/$BRANCH" 2>/dev/null; then
    # Remote branch exists, create local tracking branch
    echo "Using remote branch: origin/$BRANCH"
    git worktree add --track -b "$BRANCH" "$WORKSTREAM_DIR" "origin/$BRANCH"
else
    # No existing branch, create new from origin/main
    echo "Creating new branch: $BRANCH (from origin/main)"
    git worktree add -b "$BRANCH" "$WORKSTREAM_DIR" origin/main
fi

# Create workstream-specific config
cat > "$WORKSTREAM_DIR/.workstream.conf" << EOF
WORKSTREAM_NAME="$NAME"
WORKSTREAM_PORT_OFFSET=$OFFSET
COMPOSE_PROJECT_NAME="lana-bank-$NAME"
EOF

# Create docker-compose.workstream.yml for port offsets
cat > "$WORKSTREAM_DIR/docker-compose.workstream.yml" << EOF
services:
  core-pg:
    ports: ["$((5433 + OFFSET)):5432"]
  keycloak-pg:
    ports: ["$((5437 + OFFSET)):5432"]
  keycloak:
    ports: ["$((8081 + OFFSET)):8080", "$((9000 + OFFSET)):9000"]
    environment:
      KC_HOSTNAME_PORT: $((8081 + OFFSET))
  oathkeeper:
    ports: ["$((4455 + OFFSET)):4455", "$((4456 + OFFSET)):4456"]
  otel-agent:
    ports: ["$((4317 + OFFSET)):4317", "$((4318 + OFFSET)):4318"]
EOF

# Create devcontainer override for this workstream
mkdir -p "$WORKSTREAM_DIR/.devcontainer"
cat > "$WORKSTREAM_DIR/.devcontainer/docker-compose.override.yml" << EOF
services:
  devcontainer:
    container_name: devcontainer-$NAME
    environment:
      - WORKSTREAM_NAME=$NAME
      - WORKSTREAM_PORT_OFFSET=$OFFSET
      - COMPOSE_PROJECT_NAME=lana-bank-$NAME
    volumes:
      - $WORKSTREAM_DIR:/workspaces/lana-bank:cached
EOF

# Pre-allow direnv if available
if command -v direnv &> /dev/null; then
    direnv allow "$WORKSTREAM_DIR" 2>/dev/null || true
fi

echo ""
echo "Workstream '$NAME' created!"
echo ""
echo "Ports (when running Tilt):"
echo "  PostgreSQL:  $((5433 + OFFSET))"
echo "  Keycloak:    $((8081 + OFFSET))"
echo "  Oathkeeper:  $((4455 + OFFSET))"
echo ""

# Open tmux window/session
if [ -n "$TMUX" ]; then
    # Already in tmux, create new window
    tmux new-window -n "$NAME" -c "$WORKSTREAM_DIR"
    echo "Opened tmux window '$NAME'"
else
    # Not in tmux, create new session and attach
    echo "Starting tmux session 'lana-$NAME'..."
    tmux new-session -d -s "lana-$NAME" -n "$NAME" -c "$WORKSTREAM_DIR"
    exec tmux attach -t "lana-$NAME"
fi
