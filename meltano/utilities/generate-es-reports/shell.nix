{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = [
    pkgs.python310
    pkgs.python310Packages.pip
    pkgs.python310Packages.setuptools
    pkgs.python310Packages.wheel
    pkgs.python310Packages.black
    pkgs.ruff

    pkgs.python310Packages.google_cloud_bigquery
    pkgs.python310Packages.google_cloud_storage
    pkgs.python310Packages.dicttoxml
    pkgs.python310Packages.google_auth
  ];

  shellHook = ''
    export PYTHONPATH=$PWD:$PYTHONPATH
  '';
}
