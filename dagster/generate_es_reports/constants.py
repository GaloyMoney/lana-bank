from pathlib import Path


class Constants:
    """Simple namespace to store constants and avoid magic vars."""

    DEFAULT_XML_SCHEMAS_PATH = Path(__file__).resolve().parent / "schemas"
