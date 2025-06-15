#!/usr/bin/env bash
set -euo pipefail

# ── Pick container engine ───────────────────────────────────────────────────────
if [[ -n "${ENGINE_DEFAULT:-}" ]]; then            # honour explicit choice
  ENGINE="$ENGINE_DEFAULT"
else                                               # otherwise prefer docker
  ENGINE=docker
fi

# ensure the binary is on PATH
if ! command -v "$ENGINE" >/dev/null 2>&1; then
  printf 'Error: requested engine "%s" not found in $PATH\n' "$ENGINE" >&2
  exit 1
fi

# ── Set host gateway mapping ────────────────────────────────────────────────────
# Docker: Use the special host-gateway value (works on Linux Docker & newer Docker Desktop)
# Podman: Use the actual gateway IP from the host network interface
if [[ "$ENGINE" == docker ]]; then
  export HOST_GATEWAY="host-gateway"
else
  # For podman, we need to find the host IP that containers can reach
  # On macOS, podman creates a bridge network with the host as gateway
  if [[ "$(uname)" == "Darwin" ]]; then
    # Get the host IP from the default route visible to host
    export HOST_GATEWAY="$(route -n get default | grep 'interface:' | head -1 | awk '{print $2}' | xargs ifconfig | grep 'inet ' | head -1 | awk '{print $2}')"
  else
    # On Linux, use the gateway from host networking
    export HOST_GATEWAY="$(ip route | grep default | head -1 | awk '{print $3}')"
  fi
  echo "Using HOST_GATEWAY=$HOST_GATEWAY for podman"
fi

# ── Pull images first (prevents concurrent map writes) ─────────────────────────
echo "Pulling Docker images..."
"$ENGINE" compose -f docker-compose.yml pull

# ── Up ──────────────────────────────────────────────────────────────────────────
echo "Starting services..."
"$ENGINE" compose -f docker-compose.yml up -d "$@"

while ! pg_isready -d pg -p 5433 -U user; do
  echo "PostgreSQL not yet ready..."
  sleep 1
done
echo "PostgreSQL ready"
