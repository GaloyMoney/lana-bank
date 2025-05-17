{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.python311
            pkgs.python311Packages.pip
          ];

          shellHook = ''
            set -e # Exit immediately if a command exits with a non-zero status.

            VENV_DIR=".venv" # Relative to the meltano project root
            INSTALL_MARKER="$VENV_DIR/.plugins_installed"

            # Create a virtual environment if it doesn't exist
            if [ ! -d "$VENV_DIR" ]; then
              echo "Creating virtual environment at $VENV_DIR..."
              ${pkgs.python311}/bin/python -m venv "$VENV_DIR"
            else
              echo "Virtual environment $VENV_DIR already exists."
            fi

            # Activate the virtual environment
            if [ -f "$VENV_DIR/bin/activate" ]; then
              echo "Activating virtual environment..."
              source "$VENV_DIR/bin/activate"
              echo "Python in activated venv: $(which python)"
              echo "Python version in activated venv: $(python --version)"
            else
              echo "ERROR: Virtual environment activation script not found at $VENV_DIR/bin/activate."
              echo "Hint: If '.venv' directory exists but is empty or corrupted, remove it and reload the shell:"
              echo "rm -rf .venv && direnv reload"
              exit 1 # Stop if venv can't be activated
            fi

            # Install meltano in the virtual environment if not already installed
            if ! command -v meltano &> /dev/null; then
              echo "Installing meltano and sqlglot into the virtual environment..."
              pip install meltano sqlglot # Ensure sqlglot is pip-installed alongside meltano
            else
              echo "Meltano already installed in the virtual environment."
              # We might still want to ensure sqlglot is there if meltano was already installed
              # but this will be handled on the first run after this change if .venv is cleared.
            fi

            # Install meltano plugins if they haven't been installed before (according to the marker)
            # This uses the 'meltano' command from the activated virtual environment.
            if [ ! -f "$INSTALL_MARKER" ]; then
              echo "Installing meltano plugins..."
              meltano install
              echo "$(date): Meltano plugins installed successfully" > "$INSTALL_MARKER"
            else
              echo "Meltano plugins already installed (marker found)."
            fi

            echo "Environment ready. You can use 'meltano' now."
          '';
        };
      }
    );
}
