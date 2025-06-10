FROM ubuntu:22.04

# Install system dependencies
RUN apt-get update && apt-get install -y \
    curl git bash ca-certificates xz-utils build-essential direnv \
    && rm -rf /var/lib/apt/lists/*

# Create vscode user, nixbld group, and nixbld user
RUN useradd -m -s /bin/bash -u 1000 vscode && \
    groupadd -g 30000 nixbld && \
    useradd -g nixbld -G nixbld -s /bin/false -c "Nix build user" -d /var/empty nixbld1

# Create /nix directory and install Nix in single-user mode
RUN mkdir -m 0755 /nix && \
    curl -L https://nixos.org/nix/install > /tmp/install-nix && \
    bash /tmp/install-nix --no-daemon && \
    rm /tmp/install-nix

# Configure Nix
RUN mkdir -p /etc/nix && \
    echo 'experimental-features = nix-command flakes' > /etc/nix/nix.conf && \
    echo 'sandbox = false' >> /etc/nix/nix.conf

# Set ownership of Nix to vscode user since we're using single-user mode
RUN chown -R vscode:vscode /nix

# Switch to vscode user and set up environment
USER vscode
WORKDIR /home/vscode

# Set up shell environment and install nix-direnv
ENV USER=vscode
RUN echo 'source /nix/var/nix/profiles/default/etc/profile.d/nix.sh' >> ~/.bashrc && \
    echo 'export USER=vscode' >> ~/.bashrc && \
    /nix/var/nix/profiles/default/bin/nix profile install nixpkgs#nix-direnv && \
    echo 'eval "$(direnv hook bash)"' >> ~/.bashrc && \
    echo 'source $HOME/.nix-profile/share/nix-direnv/direnvrc' >> ~/.bashrc && \
    echo 'cd /workspaces/lana-bank 2>/dev/null || true' >> ~/.bashrc && \
    cp ~/.bashrc ~/.profile

# Set up workspace
USER root
RUN mkdir -p /workspaces && chown vscode:vscode /workspaces
USER vscode
WORKDIR /workspaces

# Default command
CMD ["/bin/bash"]