# Lana Bank Dev Container

This devcontainer provides a consistent development environment with all required tools pre-configured.

## Prerequisites

### Memory Requirements
The devcontainer requires at least **8GB of memory** for the container runtime.

**For Podman users (macOS):**
```bash
# Stop the machine
podman machine stop

# Set memory to 8GB (or more)
podman machine set --memory 8192

# Start the machine
podman machine start
```

**For Docker Desktop users:**
- Go to Settings > Resources > Advanced
- Set Memory to at least 8GB

## Quick Start

### Using VS Code / Cursor
1. Open the repository in VS Code/Cursor
2. When prompted, click "Reopen in Container"
3. Wait for the container to build and start
4. The dev environment will automatically load via direnv

### Using CLI (with podman)
```bash
# Build and start
podman-compose -f .devcontainer/docker-compose.yml up -d devcontainer

# Enter the container
podman exec -it devcontainer_devcontainer_1 bash

# Inside the container, the nix environment loads automatically
# Or manually run:
nix develop
```

## Performance

### First-Time Setup
- The first `nix develop` run downloads all dependencies
- Takes approximately **3-5 minutes** depending on network speed
- Dependencies are cached in a Docker volume for subsequent runs

### Subsequent Runs
- `nix develop` completes in **<5 seconds** using cached dependencies
- The cache persists across container rebuilds

## Volume Persistence

The devcontainer uses named volumes to persist the Nix store:
- `lana-bank-nix-store`: Persists `/nix` (all Nix packages)
- `lana-bank-nix-profile`: Persists user's Nix profile

To reset the Nix cache (e.g., if you need a clean environment):
```bash
podman volume rm lana-bank-nix-store lana-bank-nix-profile
```

## Forwarded Ports

| Port | Service |
|------|---------|
| 5433 | PostgreSQL (core-pg) |
| 4455 | Oathkeeper proxy |
| 3000 | Frontend dev server |
| 5253 | Lana backend |

## Troubleshooting

### "nix develop" is slow
- Ensure the Nix store volume is mounted (check `podman volume ls`)
- First run will always be slow as it downloads dependencies

### Process killed / OOM errors
- Increase container runtime memory to at least 8GB
- See "Prerequisites" section above

### Git ownership errors
- The devcontainer automatically configures `git safe.directory`
- If issues persist, run inside the container:
  ```bash
  git config --global --add safe.directory /workspaces/lana-bank
  ```
