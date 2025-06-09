# Dev container with Nix support
FROM ubuntu:22.04

# Install system dependencies and Nix
RUN apt-get update && apt-get install -y \
    curl sudo git bash ca-certificates xz-utils build-essential \
    && rm -rf /var/lib/apt/lists/*

# Create vscode user
RUN useradd -m -s /bin/bash -u 1000 vscode && \
    echo "vscode ALL=(root) NOPASSWD:ALL" > /etc/sudoers.d/vscode && \
    chmod 0440 /etc/sudoers.d/vscode

# Switch to vscode user and install Nix
USER vscode
WORKDIR /home/vscode

# Install Nix and configure
RUN bash -c "sh <(curl -L https://nixos.org/nix/install) --no-daemon" && \
    mkdir -p ~/.config/nix && \
    echo "experimental-features = nix-command flakes" > ~/.config/nix/nix.conf

# Set up shell environment with automatic nix develop
RUN echo 'export USER=vscode' >> ~/.bashrc && \
    echo 'source ~/.nix-profile/etc/profile.d/nix.sh 2>/dev/null || true' >> ~/.bashrc && \
    echo 'cd /workspaces/lana-bank 2>/dev/null || true' >> ~/.bashrc && \
    echo 'if [ -f flake.nix ] && [ -z "$NIX_SHELL" ]; then' >> ~/.bashrc && \
    echo '  echo "🔧 Loading Nix development environment..."' >> ~/.bashrc && \
    echo '  eval "$(nix print-dev-env 2>/dev/null || true)"' >> ~/.bashrc && \
    echo 'fi' >> ~/.bashrc && \
    cp ~/.bashrc ~/.profile

# Install essential tools
RUN bash -c "export USER=vscode && source ~/.nix-profile/etc/profile.d/nix.sh && \
    nix profile install nixpkgs#direnv nixpkgs#git nixpkgs#vim && \
    echo 'eval \"\$(direnv hook bash)\"' >> ~/.bashrc"

# Set up workspace
USER root
RUN mkdir -p /workspaces && chown vscode:vscode /workspaces
USER vscode
WORKDIR /workspaces

# Copy and set up entrypoint
COPY .devcontainer/entrypoint.sh /usr/local/bin/entrypoint.sh
USER root
RUN chmod +x /usr/local/bin/entrypoint.sh
USER vscode

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
CMD [] 