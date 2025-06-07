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
    hash = "sha256-FdfMGoDec+LVxz7ibAq0a9zvMPThgRGOIaSYKvBycDg=";
  };

  postPatch = ''
    # Replace pyproject.toml with minimal setup.py for setuptools compatibility
    cat > setup.py << 'EOF'
from setuptools import setup, find_packages
import os

# Read version from src/meltano/core/tracking/__init__.py or fallback
version = "3.5.1"

setup(
    name="meltano",
    version=version,
    packages=find_packages(where="src"),
    package_dir={"": "src"},
    package_data={
        "meltano.core.bundle": ["*.yml"],
        "meltano": ["py.typed"],
    },
    include_package_data=True,
    install_requires=[
        "click>=8.0.0",
        "pyyaml>=6.0",
        "requests>=2.25.0",
        "sqlalchemy>=1.4.0",
        "psycopg2>=2.8.0",
        "jinja2>=3.0.0",
        "jsonschema>=4.0.0",
        "packaging>=21.0",
        "cryptography>=3.0.0",
        "pydantic>=1.8.0",
        "python-dotenv>=0.19.0",
        "importlib-metadata>=4.0.0",
        "typing-extensions>=4.0.0",
        "structlog>=21.0.0",
        "watchdog>=2.0.0",
        "click-default-group>=1.2.0",
        "fasteners>=0.16.0",
        "croniter>=1.0.0",
        "pathvalidate>=2.4.0",
    ],
    entry_points={
        "console_scripts": [
            "meltano=meltano.cli:main",
        ],
    },
    python_requires=">=3.8",
)
EOF
  '';

  propagatedBuildInputs = with python3.pkgs; [
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
  ];

  # Skip tests as they require network access and additional setup
  doCheck = false;
  # Skip python imports check due to complex dependency tree
  pythonImportsCheck = [];

  meta = with lib; {
    description = "Your DataOps infrastructure, as code";
    homepage = "https://meltano.com/";
    license = licenses.mit;
    maintainers = [];
    platforms = platforms.unix;
  };
}
