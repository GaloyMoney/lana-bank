{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = [
    pkgs.poetry
    pkgs.python311
    pkgs.python311Packages.black
    pkgs.python311Packages.ruff
  ];

  shellHook = ''
    export PYTHONPATH=$PWD:$PYTHONPATH
  '';
}
