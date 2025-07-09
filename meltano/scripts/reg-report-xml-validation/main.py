import sys
import xmlschema

def validateRegulatoryReport(args):
    report = args[0]
    schema = args[1]
    try:
        xmlschema.validate(report, schema)
    except Exception as e:
        print(f"--- Error while validating file '{report}' ---")
        print(f"Exception: {e}")

    print("Job done")


if __name__ == "__main__":
    validateRegulatoryReport(sys.argv[1:])
