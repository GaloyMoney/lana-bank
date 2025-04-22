{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = [
    pkgs.python310
    pkgs.python310Packages.pip
    pkgs.python310Packages.setuptools
    pkgs.python310Packages.wheel
    pkgs.python310Packages.black
    pkgs.python310Packages.flake8
  ];

  shellHook = ''
    export PYTHONPATH=$PWD:$PYTHONPATH
    echo "Run 'black .' to format and 'flake8 .' to lint."
  '';
}
