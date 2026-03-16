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

        # Extract PR state, latest SHA and base branch from PR info
        PR_STATE=$(echo "$PR_INFO" | ${pkgs.jq}/bin/jq -r '.state')
        LATEST_SHA=$(echo "$PR_INFO" | ${pkgs.jq}/bin/jq -r '.head.sha')
        BASE_BRANCH=$(echo "$PR_INFO" | ${pkgs.jq}/bin/jq -r '.base.ref')

        if [ "$PR_STATE" = "null" ] || [ -z "$PR_STATE" ]; then
          echo "Error: Failed to fetch PR information from GitHub"
          exit 1
        fi

        echo "PR state: $PR_STATE"

        # Check if PR is still open
        if [ "$PR_STATE" != "open" ]; then
          echo "❌ PR #$PR_NUMBER is $PR_STATE. Skipping build for closed/merged PRs."
          exit 1
        fi
        echo "✓ PR is open"

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
        set -euo pipefail

        # Get the latest version including prereleases
        LAST_PRERELEASE=$(${pkgs.cocogitto}/bin/cog get-version --include-prereleases 2>/dev/null || echo "")

        # Handle case where no versions exist
        if [ -z "$LAST_PRERELEASE" ]; then
          ${pkgs.coreutils}/bin/echo -n "0.1.0-rc.1"
          exit 0
        fi

        if ${pkgs.coreutils}/bin/echo "$LAST_PRERELEASE" | ${pkgs.gnugrep}/bin/grep -q '-'; then
          LAST_PRERELEASE_BASE=$(${pkgs.coreutils}/bin/echo "$LAST_PRERELEASE" | ${pkgs.gnused}/bin/sed 's/-.*$//')

          NEXT_BASE=$(${pkgs.cocogitto}/bin/cog bump --auto --dry-run 2>/dev/null || echo "")

          if [ "$NEXT_BASE" = "$LAST_PRERELEASE_BASE" ]; then
            # increment RC
            RC_NUM=$(${pkgs.coreutils}/bin/echo "$LAST_PRERELEASE" | ${pkgs.gnused}/bin/sed 's/.*-rc\.//')
            NEXT_RC=$((RC_NUM + 1))
            ${pkgs.coreutils}/bin/echo -n "$LAST_PRERELEASE_BASE-rc.$NEXT_RC"
          else
            # new version, cut rc.1
            ${pkgs.coreutils}/bin/echo -n "$NEXT_BASE-rc.1"
          fi
        else
          # No previous prerelease exists, cut rc.1 on new version
          ${pkgs.cocogitto}/bin/cog bump --auto --pre rc.1 --dry-run | ${pkgs.coreutils}/bin/tr -d '\n'
        fi
      '';

      update-changelog = pkgs.writeShellScriptBin "update-changelog" ''
        set -euo pipefail

        if [ -z "''${1:-}" ]; then
          echo "Usage: update-changelog <version>"
          echo "Example: update-changelog 1.2.3"
          exit 1
        fi

        VERSION="$1"
        CHANGELOG_FILE="CHANGELOG.md"

        LAST_RELEASE=$(${pkgs.cocogitto}/bin/cog get-version)

        # Prepend new content to existing CHANGELOG.md
        if [ -f "$CHANGELOG_FILE" ]; then
          ${pkgs.git-cliff}/bin/git-cliff --ignore-tags ".*rc.*" $LAST_RELEASE.. --tag $VERSION --prepend $CHANGELOG_FILE
        else
          ${pkgs.git-cliff}/bin/git-cliff -o --tag $VERSION --ignore-tags ".*rc.*"
        fi

        echo "Updated $CHANGELOG_FILE with version $VERSION"
      '';

      update-docs = pkgs.writeShellApplication {
        name = "update-docs";
        runtimeInputs = with pkgs; [
          cargo
          nodejs
          pnpm
          clang
          lld
          gnused
          gnugrep
          gnumake
          gawk
          diffutils
        ];

        text = ''
          if [ -z "''${1:-}" ]; then
            echo "Usage: update-docs <version>"
            echo "Example: update-docs 1.2.3"
            exit 1
          fi

          VERSION="$1"

          echo "=== Updating documentation for version $VERSION ==="

          # Step 1: Generate public event schemas
          echo "Generating public event schemas..."
          SQLX_OFFLINE=true cargo run --package public-events-schema --features json-schema

          # Step 2: Run docs-autogenerate (generate API docs and events docs)
          echo "Generating API and events documentation..."
          cd docs-site
          pnpm install --frozen-lockfile
          pnpm run generate-api-docs
          pnpm run generate-events-docs

          # Step 3: Create versioned docs snapshot
          echo "Creating versioned docs snapshot for $VERSION..."
          pnpm run version-docs "$VERSION"

          # Step 3b: Fix Spanish version label
          # docusaurus docs:version copies current.json (which has "Siguiente" / "Next")
          # to version-X.X.X.json — we need to replace that with the actual version number
          VERSION_I18N_FILE="i18n/es/docusaurus-plugin-content-docs/version-''${VERSION}.json"
          if [ -f "$VERSION_I18N_FILE" ]; then
            echo "Fixing Spanish version label in $VERSION_I18N_FILE..."
            ${pkgs.gnused}/bin/sed -i 's/"message": "Siguiente"/"message": "'"$VERSION"'"/' "$VERSION_I18N_FILE"
            ${pkgs.gnused}/bin/sed -i 's/"description": "The label for version current"/"description": "The label for version '"$VERSION"'"/' "$VERSION_I18N_FILE"
          fi

          # Step 4: Snapshot schemas
          echo "Snapshotting schemas for $VERSION..."
          pnpm run snapshot-schemas "$VERSION"

          # Step 5: Prune old versions (keep max 5)
          echo "Pruning old documentation versions (keeping max 5)..."
          pnpm run prune-old-versions

          cd ..

          echo "=== Documentation updated for version $VERSION ==="
        '';
      };

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
      apps.update-docs = {
        type = "app";
        program = "${update-docs}/bin/update-docs";
      };
      # Also expose as default app
      apps.default = self.apps.${system}.check-latest-commit;
      # For convenience, also provide as packages
      packages.check-latest-commit = check-latest-commit;
      packages.next-version = next-version;
      packages.wait-cachix-paths = wait-cachix-paths;
      packages.update-changelog = update-changelog;
      formatter = pkgs.alejandra;
    });
}
