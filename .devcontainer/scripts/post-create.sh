#!/usr/bin/env bash
set -euo pipefail

echo "=== Lana Bank Dev Container Post-Create Setup ==="

# Check if Nix profile is properly initialized
# When using volume mounts, the profile might be empty on first run
if [[ ! -f "$HOME/.nix-profile/etc/profile.d/nix.sh" ]]; then
    echo "Nix profile not found. Initializing Nix (volumes may be fresh)..."

    # Re-run Nix installer (it's idempotent and will set up the profile)
    curl -L https://nixos.org/nix/install | bash -s -- --no-daemon

    # Source the profile
    . "$HOME/.nix-profile/etc/profile.d/nix.sh"

    # Ensure experimental features are enabled
    mkdir -p ~/.config/nix
    echo 'experimental-features = nix-command flakes' > ~/.config/nix/nix.conf

    # Install nix-direnv
    nix profile install nixpkgs#nix-direnv

    # Setup direnv config
    mkdir -p ~/.config/direnv
    echo 'source $HOME/.nix-profile/share/nix-direnv/direnvrc' > ~/.config/direnv/direnvrc

    echo "Nix installation complete!"
else
    echo "Nix profile found. Sourcing environment..."
    . "$HOME/.nix-profile/etc/profile.d/nix.sh"
fi

# Ensure git safe.directory is configured
git config --global --add safe.directory /workspaces/lana-bank 2>/dev/null || true

# Allow direnv to load the environment
cd /workspaces/lana-bank
if [[ -f .envrc ]]; then
    echo "Allowing direnv..."
    direnv allow
fi

echo ""
echo "=== Setup Complete ==="
echo ""
echo "The Nix store is persisted in a Docker volume for faster subsequent starts."
echo ""
echo "First time setup will download dependencies (~3-5 min depending on connection)."
echo "Subsequent starts will be much faster (<5 seconds) as dependencies are cached."
echo ""
echo "To enter the development shell:"
echo "  - Wait for direnv to load automatically, OR"
echo "  - Run: nix develop"
echo ""
