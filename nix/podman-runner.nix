{ pkgs, lib, stdenv }:

let
  # Build the podman-compose runner as a derivation
  podman-compose-runner = pkgs.stdenv.mkDerivation {
          pname = "podman-compose-runner";
          version = "1.0.0";
          
          # No source needed for a wrapper script
          dontUnpack = true;
          
          buildInputs = with pkgs; [
            makeWrapper
          ];
          
          installPhase = ''
            mkdir -p $out/bin
            
            # Create the runner script that uses podman-compose directly
            cat > $out/bin/podman-compose-runner << 'EOF'
            #!/usr/bin/env bash
            set -e

            # On macOS, check if podman machine exists and start it if needed
            if [[ "$OSTYPE" == "darwin"* ]]; then
              if podman machine list --format json | jq -e '.[] | select(.Name == "podman-machine-default")' >/dev/null 2>&1; then
                # Machine exists, check if it's running
                if ! podman machine list --format json | jq -e '.[] | select(.Name == "podman-machine-default" and .Running == true)' >/dev/null 2>&1; then
                  echo "Starting podman machine..."
                  podman machine start
                fi
              else
                echo "No podman machine found. Creating and starting podman-machine-default..."
                podman machine init
                podman machine start
              fi
            else
              # On Linux, ensure podman service is running
              echo "Setting up podman on Linux..."

              # Set up runtime directory
              export XDG_RUNTIME_DIR="''${XDG_RUNTIME_DIR:-/tmp/podman-runtime-$(id -u)}"
              mkdir -p "$XDG_RUNTIME_DIR"

              # Check if podman socket already exists and is responsive
              if [[ -S "$XDG_RUNTIME_DIR/podman.sock" ]]; then
                if podman --url unix://$XDG_RUNTIME_DIR/podman.sock version >/dev/null 2>&1; then
                  echo "Existing podman socket is responsive"
                  export DOCKER_HOST="unix://$XDG_RUNTIME_DIR/podman.sock"
                else
                  echo "Existing socket not responsive, cleaning up..."
                  rm -f "$XDG_RUNTIME_DIR/podman.sock"
                fi
              fi

              # Start service if socket doesn't exist
              if [[ ! -S "$XDG_RUNTIME_DIR/podman.sock" ]]; then
                echo "Starting podman system service..."
                podman system service --time=0 unix://$XDG_RUNTIME_DIR/podman.sock &
                PODMAN_SERVICE_PID=$!

                # Wait for socket to be ready
                for i in {1..30}; do
                  if [[ -S "$XDG_RUNTIME_DIR/podman.sock" ]] && podman --url unix://$XDG_RUNTIME_DIR/podman.sock version >/dev/null 2>&1; then
                    echo "Podman socket ready"
                    break
                  fi
                  sleep 1
                done

                # Ensure we actually got a working socket
                if ! podman --url unix://$XDG_RUNTIME_DIR/podman.sock version >/dev/null 2>&1; then
                  echo "Failed to start podman service"
                  exit 1
                fi
              fi

              # Export socket for podman-compose
              export DOCKER_HOST="unix://$XDG_RUNTIME_DIR/podman.sock"
            fi

            # Use podman-compose directly (it handles the socket connection internally)
            exec podman-compose "$@"
            EOF
            
            chmod +x $out/bin/podman-compose-runner
            
            # Wrap the script with the required dependencies
            wrapProgram $out/bin/podman-compose-runner \
              --prefix PATH : ${pkgs.lib.makeBinPath [
                pkgs.podman
                pkgs.podman-compose
                pkgs.coreutils
                pkgs.bash
                pkgs.jq
              ]}
          '';
          
          meta = with pkgs.lib; {
            description = "Podman-compose runner that auto-manages podman machine on macOS";
            license = licenses.mit;
            platforms = platforms.all;
          };
        };
        
  # Alternative: Pure podman-compose without auto-start
  podman-compose-simple = pkgs.stdenv.mkDerivation {
    pname = "podman-compose-simple";
    version = "1.0.0";
    
    dontUnpack = true;
    
    buildInputs = with pkgs; [ makeWrapper ];
    
    installPhase = ''
      mkdir -p $out/bin
      ln -s ${pkgs.podman-compose}/bin/podman-compose $out/bin/podman-compose-runner
    '';
  };

in
{
  # Default package is the full runner with machine management
  podman-compose-runner = podman-compose-runner;
  
  # Simple package without machine management  
  podman-compose-simple = podman-compose-simple;
}
