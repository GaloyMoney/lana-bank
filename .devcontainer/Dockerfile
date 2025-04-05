FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y curl git xz-utils && \
    curl -L https://nixos.org/nix/install | sh -s -- --daemon && \
    . /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh && \
    nix --version

# Create vscode user and group
RUN groupadd --gid 1000 vscode && \
    useradd --uid 1000 --gid 1000 -m vscode

# Set up nix environment and permissions
ENV USER=vscode
# Update PATH to include Nix binaries for both root and vscode user
ENV PATH="/home/vscode/.nix-profile/bin:/nix/var/nix/profiles/default/bin:${PATH}"

# Set up Nix permissions for all required directories
RUN mkdir -p /nix/var/nix/{daemon-socket,profiles,db,temproots,gcroots,userpool} && \
    mkdir -p /nix/store && \
    chown -R vscode:vscode /nix && \
    chmod -R 755 /nix && \
    mkdir -p /home/vscode/.nix-profile && \
    chown -R vscode:vscode /home/vscode/.nix-profile

# Optional: enable flakes and trust vscode user
RUN echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf && \
    echo "trusted-users = root vscode" >> /etc/nix/nix.conf && \
    echo "build-users-group =" >> /etc/nix/nix.conf

WORKDIR /workspace
# COPY lana/ ./lana/
COPY flake.nix flake.lock rust-toolchain.toml ./
RUN chown vscode:vscode /workspace/*

# Create a profile script that will be sourced on container start
RUN echo '. /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh' > /etc/profile.d/nix.sh && \
    chmod +x /etc/profile.d/nix.sh && \
    echo 'if [ -e /etc/profile.d/nix.sh ]; then . /etc/profile.d/nix.sh; fi' >> /home/vscode/.bashrc && \
    echo 'if [ -e /etc/profile.d/nix.sh ]; then . /etc/profile.d/nix.sh; fi' >> /home/vscode/.zshrc && \
    chown vscode:vscode /home/vscode/.bashrc /home/vscode/.zshrc

USER vscode
