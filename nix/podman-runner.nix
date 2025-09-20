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
