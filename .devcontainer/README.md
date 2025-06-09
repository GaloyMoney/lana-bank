# Lana Bank Dev Container

A simplified dev container setup for the Lana Bank application using Nix for consistent tooling.

## Features

- **Nix-based tooling**: Uses your existing `flake.nix` for development tools
- **Service integration**: Connects to services from your main `docker-compose.yml`
- **GitHub Codespaces compatible**: Works seamlessly in the cloud
- **Local development ready**: Works with VS Code dev containers

## Getting Started

### Using GitHub Codespaces

1. Open this repository in GitHub
2. Click **Code** → **Open with Codespaces**
3. Wait for the container to build and start
4. Open a terminal and run:
   ```bash
   # Enter the Nix development environment
   nix develop
   
   # Set up the database (if needed)
   make setup-db
   
   # Start the backend server
   make run-server
   ```

### Using VS Code Dev Containers (Local)

1. Install the [Dev Containers extension](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers)
2. Open this repository in VS Code
3. When prompted, click **Reopen in Container**
4. Follow the same steps as Codespaces above

## How It Works

The dev container setup:

1. **Extends your existing services**: Uses your main `docker-compose.yml` for all services
2. **Adds a dev container**: Provides a Nix-enabled development environment
3. **Connects everything**: Sets up networking between the dev container and services

## Available Tools

- **Nix**: Full Nix with flakes support
- **Essential tools**: git, direnv, vim pre-installed via Nix
- **Your flake.nix**: All tools defined in your project's flake are available

## Environment Variables

The dev container automatically sets up:

- `DATABASE_URL`: Connection to the main PostgreSQL database
- `SQLX_OFFLINE`: Enables SQLx offline mode
- `USER`: Set to `vscode` for proper Nix functionality

## Development Workflow

```bash
# Start services (from host or in container)
make start-deps

# Enter development environment
nix develop

# Your usual development commands work
make check-code-rust
cargo nextest run
make start-admin
```

## Troubleshooting

### Nix commands fail
- Ensure you're running commands after container startup
- Try: `source ~/.nix-profile/etc/profile.d/nix.sh` if needed

### Services not accessible
- Verify services are running: `make start-deps`
- Check that you're using service names (e.g., `core-pg`) not `localhost`

### Container won't start
- Try rebuilding: Command Palette → "Dev Containers: Rebuild Container"

## Customization

### Adding new tools
Add packages to `nativeBuildInputs` in your `flake.nix`, then rebuild the container.

### VS Code settings
Edit `.devcontainer/devcontainer.json` under `customizations.vscode.settings`. 