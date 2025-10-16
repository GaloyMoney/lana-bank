# Ultra-minimal version (flake.nix)
{
  outputs = { self, nixpkgs }: {
    packages.x86_64-linux.default = 
      nixpkgs.legacyPackages.x86_64-linux.writeText "hello.txt" 
        (let msg = builtins.getEnv "HELLO_MSG"; in 
         if msg == "" then "Hello, World!" else msg);
  };
}

# Usage:
# HELLO_MSG="Hi there!" nix build --impure
# cat result
