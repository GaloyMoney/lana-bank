{
  pkgs,
  lib,
  python311,
  fetchPypi,
  writeShellScriptBin,
  stdenv,
  zlib,
  gcc,
  ...
}: let
  inherit (pkgs) dockerTools buildEnv bash coreutils gitMinimal cacert;

  python3WithOverrides = python311.override {
    packageOverrides = self: super:
      lib.mapAttrs
      (name: pkg:
        if lib.isDerivation pkg && pkg ? overridePythonAttrs
        then pkg.overridePythonAttrs (_: {doCheck = false;})
        else pkg)
      super;
  };

  meltano-unwrapped = python3WithOverrides.pkgs.buildPythonApplication rec {
    pname = "meltano";
    version = "3.7.8";
    pyproject = true;

    src = fetchPypi {
      inherit pname version;
      hash = "sha256-dwYJzgqa4pYuXR2oadf6jRJV0ZX5r+mpSE8Km9lzDLI=";
    };

    nativeBuildInputs = with python3WithOverrides.pkgs; [hatchling];

    propagatedBuildInputs = with python3WithOverrides.pkgs; [
      click
      pyyaml
      requests
      sqlalchemy
      psycopg2
      jinja2
      jsonschema
      packaging
      cryptography
      pydantic
      python-dotenv
      importlib-metadata
      typing-extensions
      structlog
      watchdog
      click-default-group
      fasteners
      croniter
      pathvalidate
      click-didyoumean
      flatten-dict
      snowplow-tracker
      pyhumps
      rich
      ruamel-yaml
      simplejson
      configobj
      gitdb
      smmap
      gitpython
      tzlocal
      psutil
      alembic
      sqlalchemy-utils
      flask
      flask-cors
      gunicorn
      uvicorn
      celery
      redis
      boto3
      google-cloud-storage
      azure-storage-blob
      atomicwrites
      smart-open
      dateparser
      anyio
      virtualenv
    ];

    doCheck = false;
    pythonImportsCheck = [];
    dontCheckRuntimeDeps = true;

    meta = {
      description = "Your DataOps infrastructure, as code";
      homepage = "https://meltano.com/";
      license = lib.licenses.mit;
      platforms = lib.platforms.unix;
    };
  };

  meltano = writeShellScriptBin "meltano" ''
    export LD_LIBRARY_PATH="${lib.makeLibraryPath [
      stdenv.cc.cc.lib
      gcc.cc.lib
      zlib
    ]}:''${LD_LIBRARY_PATH:-}"

    if [[ "$1" == "install" || "$1" == "invoke" ]]; then
      MINIMAL_PYTHONPATH="${python3WithOverrides.pkgs.virtualenv}/lib/python3.11/site-packages"
      MINIMAL_PYTHONPATH="$MINIMAL_PYTHONPATH:${python3WithOverrides.pkgs.platformdirs}/lib/python3.11/site-packages"
      MINIMAL_PYTHONPATH="$MINIMAL_PYTHONPATH:${python3WithOverrides.pkgs.distlib}/lib/python3.11/site-packages"
      MINIMAL_PYTHONPATH="$MINIMAL_PYTHONPATH:${python3WithOverrides.pkgs.filelock}/lib/python3.11/site-packages"

      exec env -u PYTHONHOME -u NIX_PYTHONPATH \
        PATH="${python3WithOverrides}/bin:$PATH" \
        PYTHONPATH="$MINIMAL_PYTHONPATH" \
        LD_LIBRARY_PATH="$LD_LIBRARY_PATH" \
        ${meltano-unwrapped}/bin/meltano "$@"
    else
      exec env LD_LIBRARY_PATH="$LD_LIBRARY_PATH" ${meltano-unwrapped}/bin/meltano "$@"
    fi
  '';

  meltanoProject =
    pkgs.runCommand "meltano-project" {
      buildInputs = [meltano pkgs.gitMinimal pkgs.cacert];
    } ''
      set -euo pipefail

      mkdir -p $out/workspace
      cp -R ${./meltano} $out/workspace/meltano
      chmod -R u+w $out/workspace/meltano
      cd $out/workspace/meltano

      export HOME=$PWD
      ${meltano}/bin/meltano install
    '';

  meltanoImageRoot = buildEnv {
    name = "meltano-image-root";
    pathsToLink = ["/bin" "/workspace"];
    paths = [
      meltano
      bash
      coreutils
      gitMinimal
      meltanoProject
    ];
  };

  meltano-image = dockerTools.buildImage {
    name = "meltano";
    tag = "latest";

    fromImage = dockerTools.pullImage {
      imageName = "ubuntu";
      imageDigest = "sha256:496a9a44971eb4ac7aa9a218867b7eec98bdef452246c037aa206c841b653e08";
      sha256 = "sha256-LYdoE40tYih0XXJoJ8/b1e/IAkO94Jrs2C8oXWTeUTg=";
      finalImageTag = "mantic-20240122";
      finalImageName = "ubuntu";
    };

    copyToRoot = meltanoImageRoot;

    config = {
      WorkingDir = "/workspace/meltano";
      Cmd = ["${meltano}/bin/meltano"];

      Env = [
        "SSL_CERT_FILE=${cacert}/etc/ssl/certs/ca-bundle.crt"
        "GIT_SSL_CAINFO=${cacert}/etc/ssl/certs/ca-bundle.crt"
      ];
    };
  };
in {
  inherit meltano meltano-image;
}
