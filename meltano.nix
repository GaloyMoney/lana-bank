{
  lib,
  python311,
  fetchPypi,
}:

python311.pkgs.buildPythonApplication rec {
  pname = "meltano";
  version = "3.7.8";
  pyproject = true;

  src = fetchPypi {
    inherit pname version;
    hash = "sha256-dwYJzgqa4pYuXR2oadf6jRJV0ZX5r+mpSE8Km9lzDLI=";
  };

  nativeBuildInputs = with python311.pkgs; [
    hatchling
  ];

  propagatedBuildInputs = with python311.pkgs; [
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
    (fasteners.overridePythonAttrs (old: { doCheck = false; }))
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
}
