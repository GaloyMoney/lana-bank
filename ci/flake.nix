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
          pnpm run version-docs -- "$VERSION"

          # Step 4: Snapshot schemas
          echo "Snapshotting schemas for $VERSION..."
          pnpm run snapshot-schemas -- "$VERSION"

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

      rebuild-nix-cache = let
        fly = pkgs.stdenv.mkDerivation rec {
          pname = "fly";
          version = "7.11.2";

          src =
            if pkgs.stdenv.isDarwin
            then
              pkgs.fetchurl {
                url = "https://github.com/concourse/concourse/releases/download/v${version}/fly-${version}-darwin-amd64.tgz";
                sha256 = "sha256-7P5KefC2ZEHWkui0SdwFS4nO7phXKpgae76ti4PTTEM=";
              }
            else
              pkgs.fetchurl {
                url = "https://github.com/concourse/concourse/releases/download/v${version}/fly-${version}-linux-amd64.tgz";
                sha256 = "sha256-CjGP6d9W2Cmair2GOutOHpYy5skdqSq+8ZmEvRkQ2OI=";
              };

          phases = ["unpackPhase" "installPhase"];

          unpackPhase = ''
            tar -xzf $src
          '';

          installPhase = ''
            mkdir -p $out/bin
            cp fly $out/bin/
            chmod +x $out/bin/fly
          '';
        };
      in
        pkgs.writeShellScriptBin "rebuild-nix-cache" ''
          set -euo pipefail

          export PATH="${pkgs.curl}/bin:$PATH"

          echo "=== Nix Cache Rebuild Script ==="

          # Set defaults
          CONCOURSE_URL="''${CONCOURSE_URL:-https://ci.galoy.io}"
          CONCOURSE_TEAM="''${CONCOURSE_TEAM:-nix-cache}"
          PIPELINE="''${PIPELINE:-lana-bank-cache}"
          JOB="''${JOB:-populate-nix-cache-pr}"
          GITHUB_ORG="''${GITHUB_ORG:-GaloyMoney}"
          GITHUB_REPO="''${GITHUB_REPO:-lana-bank}"
          BUILD_LIMIT="''${BUILD_LIMIT:-50}"

          # Check required environment variables
          if [ -z "''${CONCOURSE_USERNAME:-}" ] || [ -z "''${CONCOURSE_PASSWORD:-}" ]; then
            echo "Error: CONCOURSE_USERNAME and CONCOURSE_PASSWORD environment variables must be set"
            echo "Example: export CONCOURSE_USERNAME=galoybot"
            echo "Example: export CONCOURSE_PASSWORD=your-password"
            exit 1
          fi

          if [ -z "''${GITHUB_TOKEN:-}" ]; then
            echo "Error: GITHUB_TOKEN environment variable is not set"
            echo "This is required to query GitHub API for PR information"
            exit 1
          fi

          TARGET="cache-bot-$$"  # Unique target name using process ID

          # Cleanup on exit
          trap "rm -f ~/.flyrc.$TARGET" EXIT

          echo "Logging into Concourse..."
          echo "URL: $CONCOURSE_URL"
          echo "Team: $CONCOURSE_TEAM"
          echo "Username: $CONCOURSE_USERNAME"

          # Login to Concourse
          ${fly}/bin/fly -t "$TARGET" login \
            -c "$CONCOURSE_URL" \
            -u "$CONCOURSE_USERNAME" \
            -p "$CONCOURSE_PASSWORD" \
            -n "$CONCOURSE_TEAM" > /dev/null 2>&1

          if [ $? -ne 0 ]; then
            echo "❌ Failed to login to Concourse"
            exit 1
          fi

          echo "✓ Successfully logged into Concourse"

          # Get latest main branch commit
          echo ""
          echo "Getting latest main branch commit..."
          latest_main=$(${fly}/bin/fly -t "$TARGET" resource-versions -r $PIPELINE/repo -c 1 --json 2>/dev/null | ${pkgs.jq}/bin/jq -r '.[0].version.ref' 2>/dev/null || echo "")

          if [ -z "$latest_main" ] || [ "$latest_main" = "null" ]; then
            echo "⚠️  Could not get latest main branch commit"
            latest_main="unknown"
          else
            echo "Latest main commit: ''${latest_main:0:7}"
          fi

          # Get all open PRs from GitHub
          echo ""
          echo "Fetching open PRs from GitHub..."

          # GitHub API pagination - get all pages
          page=1
          all_prs=""

          while true; do
            echo "  Fetching page $page..."
            response=$(${pkgs.curl}/bin/curl -s -H "Authorization: token $GITHUB_TOKEN" \
              "https://api.github.com/repos/$GITHUB_ORG/$GITHUB_REPO/pulls?state=open&per_page=100&page=$page")

            # Check if response is valid JSON array
            if ! echo "$response" | ${pkgs.jq}/bin/jq -e '.' >/dev/null 2>&1; then
              echo "❌ Failed to fetch PRs from GitHub"
              exit 1
            fi

            # Check if we got any results
            pr_count=$(echo "$response" | ${pkgs.jq}/bin/jq 'length')
            if [ "$pr_count" -eq 0 ]; then
              break
            fi

            # Append to our collection
            if [ -z "$all_prs" ]; then
              all_prs="$response"
            else
              all_prs=$(echo "$all_prs" "$response" | ${pkgs.jq}/bin/jq -s 'add')
            fi

            ((page++))
          done

          # Filter out draft PRs and extract PR numbers
          echo ""
          echo "Processing PRs..."

          # Create a list of PR numbers that are open and not draft
          pr_numbers=$(echo "$all_prs" | ${pkgs.jq}/bin/jq -r '
            .[] |
            select(.draft == false) |
            .number
          ')

          # Count non-draft PRs
          pr_count=$(echo "$pr_numbers" | wc -l | tr -d ' ')
          echo "Found $pr_count open non-draft PRs"

          if [ "$pr_count" -eq 0 ] || [ -z "$pr_numbers" ]; then
            echo "No open PRs to process"
            exit 0
          fi

          # Display PR numbers
          echo "PR numbers to check: $(echo $pr_numbers | tr '\n' ' ')"

          # Convert to a set we can check against and remove from
          # Using associative array in bash
          declare -A open_prs
          for pr in $pr_numbers; do
            open_prs[$pr]=1
          done

          # Get recent builds
          echo ""
          echo "Fetching last $BUILD_LIMIT builds from $JOB..."

          builds_json=$(${fly}/bin/fly -t "$TARGET" curl "/api/v1/teams/$CONCOURSE_TEAM/pipelines/$PIPELINE/jobs/$JOB/builds?limit=$BUILD_LIMIT" 2>/dev/null)

          if [ -z "$builds_json" ]; then
            echo "❌ Failed to get builds"
            exit 1
          fi

          # Process each build
          echo ""
          echo "Processing builds..."
          retriggered_count=0

          # Temporarily disable exit on error for the build processing loop
          set +e

          # Use process substitution instead of pipe to avoid subshell
          while read -r build_data; do
            # Decode the build data
            build=$(echo "$build_data" | ${pkgs.coreutils}/bin/base64 -d)

            build_id=$(echo "$build" | ${pkgs.jq}/bin/jq -r '.id')
            build_name=$(echo "$build" | ${pkgs.jq}/bin/jq -r '.name')
            status=$(echo "$build" | ${pkgs.jq}/bin/jq -r '.status')

            # Skip non-succeeded builds
            if [ "$status" != "succeeded" ]; then
              echo "  Build #$build_name (ID: $build_id) - Status: $status - Skipping"
              continue
            fi

            # Get build resources to find PR number
            resources=$(${fly}/bin/fly -t "$TARGET" curl "/api/v1/builds/$build_id/resources" 2>/dev/null || echo "{}")

            if [ -n "$resources" ] && [ "$resources" != "null" ] && [ "$resources" != "{}" ]; then
              pr_num=$(echo "$resources" | ${pkgs.jq}/bin/jq -r '.inputs[]? | select(.name == "prs") | .version.pr' 2>/dev/null || echo "")

              if [ -n "$pr_num" ] && [ "''${open_prs[$pr_num]:-0}" = "1" ]; then
                echo ""
                echo "  Build #$build_name (ID: $build_id) is for open PR #$pr_num"

                # Retrigger the build using the correct API endpoint
                echo "  Retriggering build..."

                result=$(${fly}/bin/fly -t "$TARGET" curl "/api/v1/teams/$CONCOURSE_TEAM/pipelines/$PIPELINE/jobs/$JOB/builds/$build_name" -- -X POST 2>/dev/null || echo "")

                if [ -n "$result" ]; then
                  # Check if we got a valid JSON response with an ID
                  new_build_id=$(echo "$result" | ${pkgs.jq}/bin/jq -r '.id' 2>/dev/null || echo "")
                  new_build_name=$(echo "$result" | ${pkgs.jq}/bin/jq -r '.name' 2>/dev/null || echo "")

                  if [ -n "$new_build_id" ] && [ "$new_build_id" != "null" ]; then
                    echo "  ✓ Retriggered as build #$new_build_name (ID: $new_build_id)"

                    # Add comment to the new build with JSON format including main hash
                    comment_text="Auto-retriggered: Ensuring cache is fresh for open PR #$pr_num (main: ''${latest_main:0:7})"
                    comment_json=$(${pkgs.jq}/bin/jq -n --arg comment "$comment_text" '{"comment": $comment}')
                    ${fly}/bin/fly -t "$TARGET" curl "/api/v1/builds/$new_build_id/comment" -- \
                      -X PUT \
                      -H "Content-Type: application/json" \
                      -d "$comment_json" >/dev/null 2>&1 || true

                    # Increment counter
                    retriggered_count=$((retriggered_count + 1))

                    # Remove this PR from our list so we don't retrigger it again
                    unset open_prs[$pr_num]
                  else
                    echo "  ❌ Failed to get new build ID from retrigger response"
                    echo "  Response: $result"
                  fi
                else
                  echo "  ❌ Failed to retrigger build"
                fi
              fi
            fi
          done < <(echo "$builds_json" | ${pkgs.jq}/bin/jq -r '.[] | @base64')

          # Re-enable exit on error
          set -e

          # Report any PRs we didn't find builds for
          remaining_prs=""
          for pr in "''${!open_prs[@]}"; do
            if [ "''${open_prs[$pr]}" = "1" ]; then
              remaining_prs="$remaining_prs $pr"
            fi
          done

          if [ -n "$remaining_prs" ]; then
            echo ""
            echo "PRs without recent successful builds:$remaining_prs"
          fi

          echo ""
          echo "=== Summary ==="
          echo "✓ Found $pr_count open non-draft PRs"
          echo "✓ Checked $BUILD_LIMIT most recent builds"
          echo "✓ Retriggered $retriggered_count builds"
          if [ -n "$remaining_prs" ]; then
            echo "⚠ Some PRs had no recent successful builds to retrigger"
          fi
          echo ""
          echo "Done!"
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
      apps.rebuild-nix-cache = {
        type = "app";
        program = "${rebuild-nix-cache}/bin/rebuild-nix-cache";
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
      packages.rebuild-nix-cache = rebuild-nix-cache;
      packages.update-changelog = update-changelog;
      formatter = pkgs.alejandra;
    });
}
