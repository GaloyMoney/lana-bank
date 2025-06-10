FROM nixos/nix:latest

# Install basic tools and create vscode user
RUN nix-env -iA \
    nixpkgs.shadow \
    nixpkgs.git \
    nixpkgs.direnv \
    nixpkgs.cacert && \
    useradd -m -s /bin/bash vscode && \
    mkdir -p /nix/var/nix/profiles/per-user/vscode && \
    chown -R vscode:vscode /nix/var/nix/profiles/per-user/vscode

# Switch to vscode user
USER vscode
WORKDIR /home/vscode

# Install nix-direnv for the vscode user
RUN nix-env -iA nixpkgs.nix-direnv

# Configure direnv to use nix-direnv
RUN mkdir -p ~/.config/direnv && \
    echo 'source $HOME/.nix-profile/share/nix-direnv/direnvrc' > ~/.config/direnv/direnvrc

# Add direnv hook to bash
RUN echo 'eval "$(direnv hook bash)"' >> ~/.bashrc

# Set environment variables
ENV USER=vscode
ENV NIX_PATH=nixpkgs=channel:nixos-unstable

WORKDIR /workspaces/lana-bank
