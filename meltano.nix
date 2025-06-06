{
  lib,
  python3,
  fetchPypi,
}:

python3.pkgs.buildPythonApplication rec {
  pname = "meltano";
  version = "3.5.1";
  format = "setuptools";

  src = fetchPypi {
    inherit pname version;
    hash = "sha256-0f3hfbq2m654467130g1yhqfzp3bnh56rqiyqzay4wyyh0dcrmqm";
  };

  propagatedBuildInputs = with python3.pkgs; [
    click
    pyyaml
    requests
    sqlalchemy
    psycopg2
    jinja2
    jsonschema
    packaging
    setuptools
    wheel
    pip
    virtualenv
    cryptography
    pydantic
    python-dotenv
    importlib-metadata
    typing-extensions
    backports-zoneinfo
    flatten-dict
    snowplow-tracker
    structlog
    pyhumps
    watchdog
    click-default-group
    fasteners
    croniter
    pathvalidate
  ];

  # Skip tests as they require network access and additional setup
  doCheck = false;

  meta = with lib; {
    description = "Your DataOps infrastructure, as code";
    homepage = "https://meltano.com/";
    license = licenses.mit;
    maintainers = [];
    platforms = platforms.unix;
  };
}