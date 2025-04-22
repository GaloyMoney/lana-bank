{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = [
    pkgs.python311
    pkgs.python311Packages.pip
    pkgs.python311Packages.setuptools
    pkgs.python311Packages.wheel
    pkgs.python311Packages.black
    pkgs.python311Packages.google-cloud-bigquery
    pkgs.python311Packages.google-cloud-storage
    pkgs.python311Packages.dicttoxml
    pkgs.python311Packages.google-auth

    pkgs.ruff
  ];

  shellHook = ''
    export PYTHONPATH=$PWD:$PYTHONPATH
  '';
}
