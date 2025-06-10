# ────────────────────────────────────────────────────────────────────
# Base system
# ────────────────────────────────────────────────────────────────────
FROM ubuntu:22.04

RUN apt-get update && \
    apt-get install -y \
      curl git bash ca-certificates xz-utils \
      build-essential direnv \
    && rm -rf /var/lib/apt/lists/*

# ────────────────────────────────────────────────────────────────────
# Non-root user and Nix store
# ────────────────────────────────────────────────────────────────────
RUN useradd -m -s /bin/bash -u 1000 vscode
RUN mkdir -m 0755 /nix && chown vscode:vscode /nix

USER vscode
WORKDIR /home/vscode

# ────────────────────────────────────────────────────────────────────
# Nix (single-user, no daemon)
# ────────────────────────────────────────────────────────────────────
RUN curl -L https://nixos.org/nix/install | bash -s -- --no-daemon

# Ensure future outputs are 755/644 instead of 775/664
RUN echo 'umask 022' >> ~/.profile

# ────────────────────────────────────────────────────────────────────
# Nix configuration (user-scoped)
# ────────────────────────────────────────────────────────────────────
RUN mkdir -p ~/.config/nix && \
    echo 'experimental-features = nix-command flakes' >> ~/.config/nix/nix.conf

# ────────────────────────────────────────────────────────────────────
# Shell helpers and nix-direnv
# ────────────────────────────────────────────────────────────────────
RUN echo 'source $HOME/.nix-profile/etc/profile.d/nix.sh' >> ~/.bashrc && \
    bash -c 'source $HOME/.nix-profile/etc/profile.d/nix.sh && \
             nix profile install nixpkgs#nix-direnv' && \
    mkdir -p ~/.config/direnv && \
    echo 'source $HOME/.nix-profile/share/nix-direnv/direnvrc' > ~/.config/direnv/direnvrc && \
    echo 'eval "$(direnv hook bash)"' >> ~/.bashrc && \
    echo 'cd /workspaces/lana-bank 2>/dev/null || true' >> ~/.bashrc

# ────────────────────────────────────────────────────────────────────
# Workspace
# ────────────────────────────────────────────────────────────────────
USER root
RUN mkdir -p /workspaces && chown vscode:vscode /workspaces
USER vscode
WORKDIR /workspaces

CMD ["/bin/bash"]
