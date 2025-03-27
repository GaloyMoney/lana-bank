FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y curl git xz-utils && \
    curl -L https://nixos.org/nix/install | sh -s -- --daemon && \
    . /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh && \
    nix --version

# Create vscode user and group
RUN groupadd --gid 1000 vscode && \
    useradd --uid 1000 --gid 1000 -m vscode

# Set up nix environment
ENV USER=root
ENV PATH="/root/.nix-profile/bin:/nix/var/nix/profiles/default/bin:${PATH}"

# Optional: enable flakes
RUN echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf

WORKDIR /workspace
# COPY lana/ ./lana/
COPY flake.nix flake.lock rust-toolchain.toml ./

# Build the flake dependencies
RUN . /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh && \
    nix develop --command echo "Nix dependencies installed successfully"

# Set proper permissions for the workspace
RUN chown vscode:vscode /workspace
