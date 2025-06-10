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

# Switch to vscode user and install Nix with proper permissions
USER vscode
WORKDIR /home/vscode

# Set USER environment variable before Nix installation
ENV USER=vscode

# Install Nix in single-user mode with explicit ownership
RUN bash -c "sh <(curl -L https://nixos.org/nix/install) --no-daemon" && \
    mkdir -p ~/.config/nix && \
    echo "experimental-features = nix-command flakes" > ~/.config/nix/nix.conf && \
    echo "sandbox = false" >> ~/.config/nix/nix.conf

# Ensure proper ownership of Nix directories
USER root
RUN if [ -d /nix ]; then \
        chown -R vscode:vscode /nix && \
        chmod -R u+w /nix; \
    fi

USER vscode

# Set up shell environment
RUN echo 'export USER=vscode' >> ~/.bashrc && \
    echo 'if [ -e ~/.nix-profile/etc/profile.d/nix.sh ]; then source ~/.nix-profile/etc/profile.d/nix.sh; fi' >> ~/.bashrc && \
    echo 'cd /workspaces/lana-bank 2>/dev/null || true' >> ~/.bashrc && \
    cp ~/.bashrc ~/.profile

# Install essential tools with proper user context
RUN bash -c "source ~/.nix-profile/etc/profile.d/nix.sh && \
    nix profile install nixpkgs#direnv nixpkgs#git nixpkgs#vim && \
    echo 'eval \"\$(direnv hook bash)\"' >> ~/.bashrc"

# Set up workspace
USER root
RUN mkdir -p /workspaces && chown vscode:vscode /workspaces
USER vscode
WORKDIR /workspaces

# Default command
CMD ["/bin/bash"] 