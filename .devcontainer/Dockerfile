# ─────────────────────────────────────────────────────────────
# 1. Base system
# ─────────────────────────────────────────────────────────────
FROM ubuntu:22.04

RUN apt-get update && \
    apt-get install -y \
    curl git bash ca-certificates xz-utils \
    build-essential direnv \
    && rm -rf /var/lib/apt/lists/*

# ─────────────────────────────────────────────────────────────
# 2. Non-root user and /nix
# ─────────────────────────────────────────────────────────────
RUN useradd -m -s /bin/bash -u 1000 vscode
RUN mkdir -m 0755 /nix && chown vscode:vscode /nix

USER vscode
WORKDIR /home/vscode

# ─────────────────────────────────────────────────────────────
# 3. Nix (single-user, no daemon)
# ─────────────────────────────────────────────────────────────
RUN curl -L https://nixos.org/nix/install | bash -s -- --no-daemon

# make nix visible to all subsequent RUN steps
ENV PATH=/home/vscode/.nix-profile/bin:/home/vscode/.nix-profile/sbin:$PATH

# ─────────────────────────────────────────────────────────────
# 4. Nix configuration and nix-direnv
# ─────────────────────────────────────────────────────────────
RUN mkdir -p ~/.config/nix && \
    echo 'experimental-features = nix-command flakes' >> ~/.config/nix/nix.conf

# source the profile script with an **absolute** path, then install nix-direnv
RUN . /home/vscode/.nix-profile/etc/profile.d/nix.sh && \
    nix profile install nixpkgs#nix-direnv

# ─────────────────────────────────────────────────────────────
# 5. Shell helpers and git configuration
# ─────────────────────────────────────────────────────────────
RUN echo '. /home/vscode/.nix-profile/etc/profile.d/nix.sh' >> ~/.bashrc && \
    mkdir -p ~/.config/direnv && \
    echo 'source $HOME/.nix-profile/share/nix-direnv/direnvrc' > ~/.config/direnv/direnvrc && \
    echo 'eval "$(direnv hook bash)"' >> ~/.bashrc && \
    echo 'umask 022' >> ~/.bashrc && \
    echo 'cd /workspaces/lana-bank 2>/dev/null || true' >> ~/.bashrc && \
    # Mark workspace as safe for git (needed for volume-mounted repos)
    git config --global --add safe.directory /workspaces/lana-bank

# ─────────────────────────────────────────────────────────────
# 6. Workspace
# ─────────────────────────────────────────────────────────────
USER root
RUN mkdir -p /workspaces && chown vscode:vscode /workspaces
USER vscode
WORKDIR /workspaces

CMD ["/bin/bash"]
