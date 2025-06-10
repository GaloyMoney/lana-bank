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

# Install Nix in multi-user (daemon) mode as root
RUN bash -c "sh <(curl -L https://nixos.org/nix/install) --daemon" && \
    mkdir -p /etc/nix && \
    echo 'experimental-features = nix-command flakes' > /etc/nix/nix.conf && \
    echo 'sandbox = false' >> /etc/nix/nix.conf

# Ensure proper ownership and permissions of Nix directories
RUN chown -R root:root /nix && \
    chmod -R a-w /nix && \
    chmod 0555 /nix

# Set up shell environment for vscode user
USER vscode
RUN echo 'if [ -e /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh ]; then source /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh; fi' >> ~/.bashrc && \
    echo 'export USER=vscode' >> ~/.bashrc && \
    echo 'cd /workspaces/lana-bank 2>/dev/null || true' >> ~/.bashrc && \
    cp ~/.bashrc ~/.profile && \
    echo 'eval "$(direnv hook bash)"' >> ~/.bashrc

# Set up workspace
USER root
RUN mkdir -p /workspaces && chown vscode:vscode /workspaces
USER vscode
WORKDIR /workspaces

# Default command
CMD ["/bin/bash"]