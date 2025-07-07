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
  inherit (pkgs) dockerTools buildEnv bash coreutils gitMinimal cacert postgresql;

  python3WithOverrides =
    (python311.override {
      packageOverrides = self: super:
        lib.mapAttrs
        (name: pkg:
          if lib.isDerivation pkg && pkg ? overridePythonAttrs
          then pkg.overridePythonAttrs (_: {doCheck = false;})
          else pkg)
        super;
    })
  .withPackages
    (ps:
      with ps; [
        virtualenv
      ]);

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

  airflow-scheduler-healthcheck = writeShellScriptBin "airflow-scheduler-healthcheck" ''
    set -euo pipefail
    DB_CONN="''${AIRFLOW__DATABASE__SQL_ALCHEMY_CONN//+psycopg2/}"
    if psql "''${DB_CONN}" -t -c "SELECT (EXTRACT(EPOCH FROM (NOW() - latest_heartbeat)) < 60) FROM job WHERE job_type='SchedulerJob' ORDER BY latest_heartbeat DESC LIMIT 1;" | grep -q 't'; then
      echo "Scheduler is healthy"
      exit 0
    else
      echo "Scheduler heartbeat is stale"
      exit 1
    fi
  '';

  meltanoEntrypoint = writeShellScriptBin "meltano-entrypoint" ''
    set -euo pipefail
    ln -sf /usr/bin/sh /bin/sh

    echo "Starting Meltano entrypoint..."

    # Check if /meltano directory exists (indicating a volume mount)
    if [ -d "/meltano" ]; then
        echo "Volume mount detected at /meltano - setting up fresh environment..."

        # Remove existing workspace content except .meltano (to preserve any existing plugins)
        cd /workspace/meltano
        find . -maxdepth 1 -not -name '.meltano' -not -name 'keyfile.json' -not -name '.' -not -name '..' -exec rm -rf {} +

        # Copy all content from mounted volume (excluding .meltano if it exists)
        echo "Copying files from volume mount..."
        cd /meltano
        find . -maxdepth 1 -not -name '.meltano' -not -name 'keyfile.json' -not -name '.' -not -name '..' -exec cp -r {} /workspace/meltano/ \;

        # Go back to workspace and ensure permissions
        cd /workspace/meltano
        find . -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true
        find . -name "*.pyc" -delete 2>/dev/null || true
        find . -name "*.pyo" -delete 2>/dev/null || true
        find . -not -name 'keyfile.json' -exec chmod u+w {} +

        # Fresh install of all plugins
        echo "Running fresh meltano install..."
        ${meltano}/bin/meltano install

        echo "Volume mount setup complete"
    else
        echo "No volume mount detected - using pre-built image with installed plugins"
    fi

    # Change to meltano workspace
    cd /workspace/meltano

    # Execute the original command
    echo "Executing command: $*"
    exec "$@"
  '';

  srcRoot = toString ./meltano;
  ignoreRE = builtins.match;
  relPath = p: lib.removePrefix srcRoot (toString p);

  meltanoSrc = lib.cleanSourceWith {
    src = ./meltano;

    filter = path: _type: let
      p = relPath path;
    in
      ! (
        ignoreRE "^(/\\.git(/|$))" p
        != null
        || ignoreRE "^(/venv(/|$))" p != null
        || ignoreRE "^(/\\.meltano(/|$))" p != null
        || ignoreRE "^(/\\.env$)" p != null
        || ignoreRE "^(/ui\\.cfg$)" p != null
        || ignoreRE "^(/output(/|$))" p != null
        || ignoreRE "^(/transform/(target|dbt_modules|logs)(/|$))" p != null
      );
  };

  meltanoProject =
    pkgs.runCommand "meltano-project" {
      buildInputs = [meltano pkgs.gitMinimal pkgs.cacert];
    } ''
      set -euo pipefail
      mkdir -p $out/workspace
      cp -R ${meltanoSrc} $out/workspace/meltano
      chmod -R u+w $out/workspace/meltano

      cd $out/workspace/meltano
      meltano install
    '';

  meltanoImageRoot = buildEnv {
    name = "meltano-image-root";
    pathsToLink = ["/bin" "/workspace"];
    paths = [
      meltano
      meltanoProject
      meltanoEntrypoint
      airflow-scheduler-healthcheck
      postgresql
      bash
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
    compressor = "none";

    config = {
      WorkingDir = "/workspace/meltano";
      Entrypoint = ["${meltanoEntrypoint}/bin/meltano-entrypoint"];

      Env = [
        "SSL_CERT_FILE=${cacert}/etc/ssl/certs/ca-bundle.crt"
        "GIT_SSL_CAINFO=${cacert}/etc/ssl/certs/ca-bundle.crt"
        "GOOGLE_APPLICATION_CREDENTIALS=/workspace/meltano/keyfile.json"
        "DBT_BIGQUERY_KEYFILE=/workspace/meltano/keyfile.json"
        "TARGET_BIGQUERY_LOCATION=US"
      ];
    };
  };
in {
  inherit meltano meltano-image;
}
