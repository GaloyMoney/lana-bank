from airflow.plugins_manager import AirflowPlugin
from flask import Blueprint, jsonify, request
from google.cloud import storage
from google.oauth2 import service_account
import os
from datetime import datetime, timedelta
from collections import defaultdict

# Create Flask Blueprint
reports_bp = Blueprint("reports_api", __name__, url_prefix="/api/v1")

def get_storage_client():
    """Initialize Google Cloud Storage client"""
    keyfile = os.getenv("GOOGLE_APPLICATION_CREDENTIALS")
    if not keyfile or not os.path.isfile(keyfile):
        raise RuntimeError(
            "GOOGLE_APPLICATION_CREDENTIALS environment variable must be set to the path of a valid service account JSON file."
        )
    
    project_id = os.getenv("DBT_BIGQUERY_PROJECT")
    credentials = service_account.Credentials.from_service_account_file(keyfile)
    return storage.Client(project=project_id, credentials=credentials)

def get_bucket():
    """Get the GCS bucket for reports"""
    bucket_name = os.getenv("DOCS_BUCKET_NAME")
    if not bucket_name:
        raise RuntimeError("DOCS_BUCKET_NAME environment variable must be set")
    
    storage_client = get_storage_client()
    return storage_client.bucket(bucket_name)

def parse_report_blob(blob_name):
    """Parse blob name to extract date and report name"""
    # Expected format: reports/2025-06-29/nrsf_03/funcionarios_y_empleados.txt
    parts = blob_name.split('/')
    if len(parts) != 4 or parts[0] != 'reports':
        return None
    
    try:
        report_date = parts[1]  # 2025-06-29
        report_category = parts[2]  # nrsf_03
        report_file = parts[3]  # funcionarios_y_empleados.txt
        report_name = report_file.rsplit('.', 1)[0]  # funcionarios_y_empleados
        
        # Validate date format
        datetime.strptime(report_date, '%Y-%m-%d')
        
        return {
            'date': report_date,
            'report_name': report_name,
            'report_category': report_category,
            'blob_name': blob_name,
            'filename': report_file
        }
    except ValueError:
        return None


@reports_bp.route("/reports/dates", methods=["GET"])
def get_available_dates():
    """
    Return all dates for which reports are available
    
    Response format:
    {
        "dates": [
            "2024-01-15",
            "2024-01-14",
            "2024-01-13",
            ...
        ],
        "total_count": 3
    }
    """
    try:
        bucket = get_bucket()

        # List all blobs in the reports/ prefix
        blobs = bucket.list_blobs(prefix='reports/')
        dates = set()

        for blob in blobs:
            parsed = parse_report_blob(blob.name)
            if not parsed:
                continue

            dates.add(parsed['date'])

        # Convert to sorted list (newest first)
        sorted_dates = sorted(list(dates), reverse=True)

        return jsonify({
            'dates': sorted_dates,
            'total_count': len(sorted_dates)
        })

    except Exception as e:
        return jsonify({'error': f'Error fetching available dates: {str(e)}'}), 500

@reports_bp.route("/reports/date/<date>", methods=["GET"])
def get_reports_by_date(date):
    """
    Return signed URLs of all reports for a given date
    
    Args:
        date: Date in YYYY-MM-DD format (e.g., "2024-01-15")
    
    Query Parameters:
        expiration_hours: Hours until signed URLs expire (default: 1)
    
    Response format:
    {
        "date": "2025-06-29",
        "reports": [
            {
                "report_name": "funcionarios_y_empleados",
                "report_category": "nrsf_03",
                "filename": "funcionarios_y_empleados.txt",
                "signed_url": "https://storage.googleapis.com/...",
                "expires_at": "2025-06-29T15:30:00Z",
                "size_bytes": 1024,
                "created": "2025-06-29T10:00:00Z"
            },
            ...
        ],
        "total_count": 2
    }
    """
    try:
        # Validate date format
        try:
            datetime.strptime(date, '%Y-%m-%d')
        except ValueError:
            return jsonify({
                'error': 'Invalid date format. Expected format: YYYY-MM-DD'
            }), 400

        # Get expiration hours from query parameters
        expiration_hours = request.args.get('expiration_hours', 1, type=int)
        if expiration_hours < 1 or expiration_hours > 168:  # Max 1 week
            return jsonify({
                'error': 'expiration_hours must be between 1 and 168 (1 week)'
            }), 400

        bucket = get_bucket()

        # List all blobs for the specific date
        date_prefix = f'reports/{date}/'
        blobs = bucket.list_blobs(prefix=date_prefix)

        reports = []
        expiration_time = datetime.utcnow() + timedelta(hours=expiration_hours)

        for blob in blobs:
            parsed = parse_report_blob(blob.name)
            if not parsed or parsed['date'] != date:
                continue

            # Generate signed URL
            signed_url = blob.generate_signed_url(
                expiration=expiration_time,
                method='GET'
            )

            reports.append({
                'uri': blob.name,
                'report_name': parsed['report_name'],
                'report_category': parsed['report_category'],
                'filename': parsed['filename'],
                'signed_url': signed_url,
                'expires_at': expiration_time.isoformat() + 'Z',
                'size_bytes': blob.size,
                'created': blob.time_created.isoformat() if blob.time_created else None
            })

        # Sort by report name
        reports.sort(key=lambda x: x['report_name'])

        return jsonify({
            'date': date,
            'reports': reports,
            'total_count': len(reports)
        })
    except Exception as e:
        return jsonify({'error': f'Error fetching reports for date {date}: {str(e)}'}), 500

@reports_bp.route("/reports/health", methods=["GET"])
def health_check():
    """Health check endpoint for the reports API"""
    try:
        # Test GCS connection
        bucket = get_bucket()
        bucket.exists()  # This will raise an exception if there are auth issues

        return jsonify({
            'status': 'healthy',
            'timestamp': datetime.utcnow().isoformat() + 'Z',
            'bucket': os.getenv('DOCS_BUCKET_NAME')
        })
    except Exception as e:
        return jsonify({
            'status': 'unhealthy',
            'error': str(e),
            'timestamp': datetime.utcnow().isoformat() + 'Z'
        }), 500

class ReportsApiPlugin(AirflowPlugin):
    """Airflow plugin to expose reports API endpoints"""
    name = "reports_api"
    flask_blueprints = [reports_bp]
