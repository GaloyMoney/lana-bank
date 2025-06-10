# Use an official Nix image
# No build stages needed anymore if user is created at runtime
FROM nixos/nix:latest

# Configure Nix system-wide for flakes and trusted users
# We still need root and potentially vscode (if created later) as trusted
RUN mkdir -p /etc/nix && \
    echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf && \
    # Trust root initially. The onCreateCommand will need to run as root.
    # If vscode user is created, it might need adding here too depending on permissions.
    echo "trusted-users = root vscode" >> /etc/nix/nix.conf && \
    echo "build-users-group =" >> /etc/nix/nix.conf

# Set up the workspace directory
WORKDIR /workspace
# Copy flake files and toolchain definition first for layer caching
COPY flake.nix flake.lock rust-toolchain.toml ./
# Workspace ownership will be handled by devcontainer features or onCreateCommand

# Pre-build the Nix development environment dependencies
# This ensures all packages are downloaded/built into the image layer
# Determine system dynamically for cross-platform compatibility
RUN SYSTEM=$(nix eval --raw nixpkgs#system) && \
    nix build .#devShells.${SYSTEM}.default --no-link

# Run as root by default
# USER vscode # Removed - user creation moved to runtime

# Optional: Command to verify setup or keep container running
CMD ["sleep", "infinity"]
