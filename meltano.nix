# meltano.nix
{
  lib,
  pkgs,
  python3,
  python3Packages,
  mkShell,
  devEnvVars,
}: let
  # Use Python 3.11 specifically for Airflow compatibility
  python311 = pkgs.python311;
  python311Packages = pkgs.python311Packages;

  # Define Python environment with necessary packages using Python 3.11
  pythonEnv = python311.withPackages (ps:
    with ps; [
      pip
      virtualenv
      # Add other Python packages you need here
    ]);

  # Define the shell that will be used for Meltano development
  meltanoShell = mkShell {
    # Include the Python environment and other necessary packages
    packages = [
      pythonEnv
    ];

    # Environment variables inherited from flake.nix instead of hardcoded
    inherit (devEnvVars) PGDATABASE PGUSER PGPASSWORD PGHOST;

    # Shell hook to set up the virtual environment and install Meltano
    shellHook = ''
      echo "🚀 Entering Meltano development environment (Python 3.11)"
      MELTANO_PROJECT_DIR="./meltano"
      VENV_DIR="$MELTANO_PROJECT_DIR/.venv"
      INSTALL_MARKER="$MELTANO_PROJECT_DIR/.plugins_installed"

      # Check if virtual env exists and check Python version
      if [ -d "$VENV_DIR" ]; then
        # Activate the venv to check Python version
        source "$VENV_DIR/bin/activate" 2>/dev/null || true
        CURRENT_PYTHON_VERSION=$(python --version 2>&1 | grep -oE 'Python 3\.([0-9]+)')
        deactivate 2>/dev/null || true

        # If not Python 3.11.x, remove the venv to recreate it
        if [[ "$CURRENT_PYTHON_VERSION" != "Python 3.11" ]]; then
          echo "Found $CURRENT_PYTHON_VERSION venv, but Python 3.11 is required for Airflow"
          echo "Removing existing virtual environment to create a new one with Python 3.11..."
          rm -rf "$VENV_DIR"
          # Also remove the marker since we're recreating the environment
          rm -f "$INSTALL_MARKER"
        fi
      fi

      # Create and activate venv if needed
      if [ ! -d "$VENV_DIR" ]; then
        echo "Creating Python virtual environment in $VENV_DIR using ${python311}/bin/python3..."
        ${python311}/bin/python3 -m venv "$VENV_DIR"
        echo "Virtual environment created with Python 3.11."
      fi

      # Activate the venv
      if [ -f "$VENV_DIR/bin/activate" ]; then
        echo "Activating virtual environment..."
        source "$VENV_DIR/bin/activate"

        # Verify Python version
        VENV_PYTHON_VERSION=$(python --version)
        echo "Virtual environment Python: $VENV_PYTHON_VERSION"

        # Check if meltano is installed in the venv
        if ! command -v meltano >/dev/null 2>&1; then
          echo "Installing Meltano in virtual environment..."
          pip install meltano
        fi

        echo "Virtual environment activated with Meltano $(meltano --version 2>/dev/null || echo 'not available')"

        # Run meltano install only if it hasn't been run before (check marker file)
        if [ ! -f "$INSTALL_MARKER" ]; then
          echo "-------------------------------------------------------------"
          echo "Running 'meltano install' to install plugins (first-time setup)"
          echo "-------------------------------------------------------------"
          if (cd "$MELTANO_PROJECT_DIR" && meltano install); then
            # Mark installation as complete
            echo "$(date): Meltano plugins installed successfully" > "$INSTALL_MARKER"
            echo "Plugins installed successfully. This will not be repeated on future shell entries."
          else
            echo "Warning: meltano install failed. Will try again next time."
          fi
          echo "-------------------------------------------------------------"
        else
          echo "-------------------------------------------------------------"
          echo "Meltano plugins already installed. Skipping installation."
          echo "Last installed: $(cat "$INSTALL_MARKER")"
          echo "To force reinstallation, remove $INSTALL_MARKER"
          echo "-------------------------------------------------------------"
        fi

      else
        echo "Warning: Virtual environment activation script not found at $VENV_DIR/bin/activate"
      fi

      echo "Meltano setup complete. You can now use 'meltano' commands."
    '';
  };
in
  meltanoShell
