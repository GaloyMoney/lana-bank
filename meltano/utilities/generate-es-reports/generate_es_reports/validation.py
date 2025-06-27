import importlib.resources
from xmlschema import XMLSchema

class Validator:
    def __init__(self):
        self.xsds = {}
        with importlib.resources.open_text(
            "generate_es_reports.schemas.CTRI", "ssf_ctri_persona.xsd"
        ) as f:
            self.xsds["persona"] = XMLSchema(f)

    def validate(self, report_name, report_bytes):
        if report_name not in self.xsds:
            print(f"No validator for {report_name}")
            return
        try:
            self.xsds[report_name].validate(report_bytes)
        except Exception as e:
            print(f"--- Error while validating file '{report_name}' ---")
            print(f"Exception: {e}")
