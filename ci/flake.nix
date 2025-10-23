{
  description = "Check if current commit is the latest in PR";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
      check-latest-commit = pkgs.writeShellScriptBin "check-latest-commit" ''
        set -euo pipefail
        # Check required environment variables
        if [ -z "''${GITHUB_TOKEN:-}" ]; then
          echo "Error: GITHUB_TOKEN environment variable is not set"
          exit 1
        fi
        if [ -z "''${GITHUB_ORG:-}" ] || [ -z "''${GITHUB_REPO:-}" ]; then
          echo "Error: GITHUB_ORG and GITHUB_REPO environment variables must be set"
          echo "Example: GITHUB_ORG=myorg GITHUB_REPO=myrepo"
          exit 1
        fi

        # Get current commit SHA
        HEAD_SHA=$(cat .git/resource/head_sha)
        echo "Head sha of the PR: $HEAD_SHA"

        # Get base SHA
        if [ -f .git/resource/base_sha ]; then
          BASE_SHA=$(cat .git/resource/base_sha)
          echo "Base sha of the PR: $BASE_SHA"
        else
          echo "Warning: .git/resource/base_sha file not found, skipping base check"
          BASE_SHA=""
        fi

        # Extract PR number from .git/resource/pr
        if [ ! -f .git/resource/pr ]; then
          echo "Error: .git/resource/pr file not found"
          exit 1
        fi
        PR_NUMBER=$(cat .git/resource/pr)
        echo "PR number: $PR_NUMBER"

        # Get the PR information from GitHub API
        PR_INFO=$(${pkgs.curl}/bin/curl -s -H "Authorization: token $GITHUB_TOKEN" \
          "https://api.github.com/repos/$GITHUB_ORG/$GITHUB_REPO/pulls/$PR_NUMBER")

        # Extract latest SHA and base branch from PR info
        LATEST_SHA=$(echo "$PR_INFO" | ${pkgs.jq}/bin/jq -r '.head.sha')
        BASE_BRANCH=$(echo "$PR_INFO" | ${pkgs.jq}/bin/jq -r '.base.ref')

        if [ "$LATEST_SHA" = "null" ] || [ -z "$LATEST_SHA" ]; then
          echo "Error: Failed to fetch PR information from GitHub"
          exit 1
        fi

        echo "Latest PR upstream SHA: $LATEST_SHA"
        echo "PR base branch: $BASE_BRANCH"

        # Check if this is the latest commit in the PR
        if [ "$HEAD_SHA" != "$LATEST_SHA" ]; then
          echo "❌ This is not the latest commit in the PR. Aborting."
          exit 1
        fi
        echo "✓ Latest commit in PR confirmed"

        # Check if base SHA is up-to-date with main branch (if base_sha exists)
        if [ -n "$BASE_SHA" ] && [ -n "$BASE_BRANCH" ]; then
          echo ""
          echo "Checking if PR base is up-to-date with $BASE_BRANCH..."

          # Get the current HEAD of the base branch
          MAIN_HEAD=$(${pkgs.curl}/bin/curl -s -H "Authorization: token $GITHUB_TOKEN" \
            "https://api.github.com/repos/$GITHUB_ORG/$GITHUB_REPO/git/refs/heads/$BASE_BRANCH" \
            | ${pkgs.jq}/bin/jq -r '.object.sha')

          if [ "$MAIN_HEAD" = "null" ] || [ -z "$MAIN_HEAD" ]; then
            echo "Error: Failed to fetch $BASE_BRANCH branch information from GitHub"
            exit 1
          fi

          echo "Current $BASE_BRANCH HEAD: $MAIN_HEAD"
          echo "PR base SHA: $BASE_SHA"

          if [ "$BASE_SHA" != "$MAIN_HEAD" ]; then
            echo "❌ PR base is outdated! The $BASE_BRANCH branch has moved forward."
            echo "   This PR needs to be rebased or the cache needs to be rebuilt."
            exit 1
          fi
          echo "✓ PR base is up-to-date with $BASE_BRANCH"
        fi

        echo ""
        echo "✅ All checks passed!"
        exit 0
      '';

      next-version = pkgs.writeShellScriptBin "next-version" ''
        # Try to get version from auto bump
        OUTPUT=$(${pkgs.cocogitto}/bin/cog bump --auto --dry-run 2>&1 || true)

        # Check if output is a valid semver (e.g., 1.2.3)
        if ${pkgs.coreutils}/bin/echo "$OUTPUT" | ${pkgs.gnugrep}/bin/grep -qE "^[0-9]+\.[0-9]+\.[0-9]+$"; then
          # Output the auto bump result (it's a valid version)
          ${pkgs.coreutils}/bin/echo "$OUTPUT" | ${pkgs.coreutils}/bin/tr -d '\n'
        else
          # Output is not a semver, default to patch bump
          ${pkgs.cocogitto}/bin/cog bump --patch --dry-run | ${pkgs.coreutils}/bin/tr -d '\n'
        fi
      '';

      wait-cachix-paths = pkgs.writeShellScriptBin "wait-cachix-paths" ''
        set +e  # Don't exit on non-zero return codes

        # Parse command line arguments
        PATHS_FILE=""
        CACHE_NAME=""
        MAX_ATTEMPTS=60
        RETRY_DELAY=10

        usage() {
          echo "Usage: $0 -p PATHS_FILE -c CACHE_NAME [-a MAX_ATTEMPTS] [-d RETRY_DELAY]"
          echo ""
          echo "Options:"
          echo "  -p PATHS_FILE    Path to file containing nix store paths (required)"
          echo "  -c CACHE_NAME    Name of the Cachix cache (required)"
          echo "  -a MAX_ATTEMPTS  Maximum number of retry attempts (default: 60)"
          echo "  -d RETRY_DELAY   Delay between retries in seconds (default: 10)"
          echo "  -h               Show this help message"
          exit 1
        }

        while getopts "p:c:a:d:h" opt; do
          case $opt in
            p) PATHS_FILE="$OPTARG" ;;
            c) CACHE_NAME="$OPTARG" ;;
            a) MAX_ATTEMPTS="$OPTARG" ;;
            d) RETRY_DELAY="$OPTARG" ;;
            h) usage ;;
            *) usage ;;
          esac
        done

        # Check required arguments
        if [ -z "$PATHS_FILE" ] || [ -z "$CACHE_NAME" ]; then
          echo "Error: Both -p and -c options are required"
          usage
        fi

        if [ ! -f "$PATHS_FILE" ]; then
          echo "Error: Paths file not found: $PATHS_FILE"
          exit 1
        fi

        echo "Waiting for all paths to be available in cache: $CACHE_NAME"
        echo "Max attempts: $MAX_ATTEMPTS, Retry delay: ''${RETRY_DELAY}s"

        attempt=1
        while [ $attempt -le $MAX_ATTEMPTS ]; do
          echo -e "\nAttempt $attempt of $MAX_ATTEMPTS"
          all_found=true
          missing_count=0

          while IFS= read -r path; do
            # Skip empty lines
            [ -z "$path" ] && continue

            # Extract hash from nix store path
            hash=$(echo "$path" | ${pkgs.gnused}/bin/sed -n 's|/nix/store/\([^-]*\).*|\1|p')

            if [ -z "$hash" ]; then
              echo "Warning: Could not extract hash from path: $path"
              continue
            fi

            url="https://''${CACHE_NAME}.cachix.org/''${hash}.narinfo"

            # Check if path exists in cache
            if ${pkgs.curl}/bin/curl -s -f -o /dev/null "$url" 2>/dev/null; then
              echo "✓ Found: $path"
            else
              echo "✗ Missing: $path"
              all_found=false
              missing_count=$((missing_count + 1))
            fi
          done < "$PATHS_FILE"

          if [ "$all_found" = true ]; then
            echo -e "\nSuccess! All paths are available in the cache."
            exit 0
          fi

          echo -e "\nStill missing $missing_count paths..."

          if [ $attempt -lt $MAX_ATTEMPTS ]; then
            echo "Waiting ''${RETRY_DELAY}s before next attempt..."
            ${pkgs.coreutils}/bin/sleep "$RETRY_DELAY"
          fi

          ((attempt++))
        done

        echo -e "\nError: Maximum attempts reached. Some paths are still not available in the cache."
        exit 1
      '';
    in {
      apps.check-latest-commit = {
        type = "app";
        program = "${check-latest-commit}/bin/check-latest-commit";
      };
      apps.next-version = {
        type = "app";
        program = "${next-version}/bin/next-version";
      };
      apps.wait-cachix-paths = {
        type = "app";
        program = "${wait-cachix-paths}/bin/wait-cachix-paths";
      };
      # Also expose as default app
      apps.default = self.apps.${system}.check-latest-commit;
      # For convenience, also provide as packages
      packages.check-latest-commit = check-latest-commit;
      packages.next-version = next-version;
      packages.wait-cachix-paths = wait-cachix-paths;
      formatter = pkgs.alejandra;
    });
}
