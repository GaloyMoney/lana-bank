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
        CURRENT_SHA=$(${pkgs.git}/bin/git rev-parse HEAD)
        echo "Current commit SHA: $CURRENT_SHA"
        # Extract PR number from .git/resource/pr
        if [ ! -f .git/resource/pr ]; then
          echo "Error: .git/resource/pr file not found"
          exit 1
        fi
        PR_NUMBER=$(cat .git/resource/pr)
        echo "PR number: $PR_NUMBER"
        # Get the latest SHA from GitHub API
        LATEST_SHA=$(${pkgs.curl}/bin/curl -s -H "Authorization: token $GITHUB_TOKEN" \
          "https://api.github.com/repos/$GITHUB_ORG/$GITHUB_REPO/pulls/$PR_NUMBER" \
          | ${pkgs.jq}/bin/jq -r '.head.sha')
        if [ "$LATEST_SHA" = "null" ] || [ -z "$LATEST_SHA" ]; then
          echo "Error: Failed to fetch PR information from GitHub"
          exit 1
        fi
        echo "Latest PR SHA: $LATEST_SHA"
        if [ "$CURRENT_SHA" != "$LATEST_SHA" ]; then
          echo "This is not the latest commit. Aborting."
          exit 1
        fi
        echo "Latest commit - confirmed"
        exit 0
      '';
      
      next-version = pkgs.writeShellScriptBin "next-version" ''
        # Try to get version from auto bump
        OUTPUT=$(${pkgs.cocogitto}/bin/cog bump --auto --dry-run 2>&1 || true)
        
        # Check if no conventional commits were found
        if echo "$OUTPUT" | grep -q "No conventional commits for your repository that required a bump"; then
          # Default to patch bump
          ${pkgs.cocogitto}/bin/cog bump --patch --dry-run | ${pkgs.coreutils}/bin/tr -d '\n'
        else
          # Output the auto bump result
          echo "$OUTPUT" | ${pkgs.coreutils}/bin/tr -d '\n'
        fi
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
      
      # Also expose as default app
      apps.default = self.apps.${system}.check-latest-commit;
      
      # For convenience, also provide as packages
      packages.check-latest-commit = check-latest-commit;
      packages.next-version = next-version;
      
      formatter = pkgs.alejandra;
    });
}
