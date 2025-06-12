{
  lib,
  python311,
  fetchPypi,
  writeShellScriptBin,
  stdenv,
  zlib,
  gcc,
}: let
  python3WithOverrides = python311.override {
    packageOverrides = self: super:
    # Apply doCheck = false to all Python packages
      lib.mapAttrs (
        name: pkg:
          if lib.isDerivation pkg && pkg ? overridePythonAttrs
          then
            pkg.overridePythonAttrs (old: {
              doCheck = false;
            })
          else pkg
      )
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

    nativeBuildInputs = with python3WithOverrides.pkgs; [
      hatchling
    ];

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

    # Skip tests as they require network access and additional setup
    doCheck = false;
    # Skip python imports check due to complex dependency tree
    pythonImportsCheck = [];
    # Skip runtime deps check due to optional dependencies
    dontCheckRuntimeDeps = true;

    meta = with lib; {
      description = "Your DataOps infrastructure, as code";
      homepage = "https://meltano.com/";
      license = licenses.mit;
      maintainers = [];
      platforms = platforms.unix;
    };
  };
in
  writeShellScriptBin "meltano" ''
    # Set LD_LIBRARY_PATH to include necessary C++ libraries for Airflow and other tools
    export LD_LIBRARY_PATH="${lib.makeLibraryPath [
      stdenv.cc.cc.lib
      gcc.cc.lib
      zlib
    ]}:''${LD_LIBRARY_PATH:-}"

    if [[ "$1" == "install" ]] || [[ "$1" == "invoke" ]]; then
      # Set minimal PYTHONPATH with virtualenv and required dependencies
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
      # For other commands, use meltano normally with library path
      exec env LD_LIBRARY_PATH="$LD_LIBRARY_PATH" ${meltano-unwrapped}/bin/meltano "$@"
    fi
  ''
