import json
import base64

from dlt.destinations import bigquery

def create_bigquery_destination(base64_credentials):
        """Create BigQuery destination with programmatic credentials configuration"""

        try:
            # Decode the base64-encoded JSON credentials
            credentials_json = base64.b64decode(base64_credentials).decode('utf-8')
            credentials = json.loads(credentials_json)
        except (base64.binascii.Error, json.JSONDecodeError) as e:
            raise ValueError(f"Failed to decode base64 credentials: {e}")
        
        required_fields = ["type", "project_id", "private_key", "client_email"]
        for field in required_fields:
            if field not in credentials:
                raise ValueError(f"Missing required field '{field}' in credentials")
        
        return bigquery(
            credentials=credentials,
            project_id=credentials["project_id"],
            location="US"  # Optional: specify location
        )