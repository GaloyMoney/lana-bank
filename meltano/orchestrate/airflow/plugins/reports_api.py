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
    # Expected format: reports/2024-01-15/report_name.xml
    parts = blob_name.split('/')
    if len(parts) != 3 or parts[0] != 'reports':
        return None
    
    try:
        report_date = parts[1]  # 2024-01-15
        report_file = parts[2]  # report_name.xml
        report_name = report_file.replace('.xml', '')  # report_name
        
        # Validate date format
        datetime.strptime(report_date, '%Y-%m-%d')
        
        return {
            'date': report_date,
            'report_name': report_name,
            'blob_name': blob_name,
            'filename': report_file
        }
    except ValueError:
        return None

def generate_report_id(date, report_name):
    """Generate a unique report ID from date and report name"""
    return f"{date}_{report_name}"

def parse_report_id(report_id):
    """Parse report ID to extract date and report name"""
    try:
        # Expected format: 2024-01-15_report_name
        parts = report_id.split('_', 1)  # Split only on first underscore
        if len(parts) != 2:
            return None
        
        date_str = parts[0]
        report_name = parts[1]
        
        # Validate date format
        datetime.strptime(date_str, '%Y-%m-%d')
        
        return {
            'date': date_str,
            'report_name': report_name
        }
    except ValueError:
        return None

@reports_bp.route("/reports", methods=["GET"])
def get_all_reports():
    """
    Return the ID and dates of all available reports
    
    Response format:
    {
        "reports": [
            {
                "id": "2024-01-15_report_sales",
                "date": "2024-01-15",
                "report_name": "report_sales"
            },
            ...
        ]
    }
    """
    try:
        bucket = get_bucket()
        
        # List all blobs in the reports/ prefix
        blobs = bucket.list_blobs(prefix='reports/')
        
        reports = []
        
        for blob in blobs:
            parsed = parse_report_blob(blob.name)
            if not parsed:
                continue
            
            report_id = generate_report_id(parsed['date'], parsed['report_name'])
            
            reports.append({
                'id': report_id,
                'date': parsed['date'],
                'report_name': parsed['report_name']
            })
        
        # Sort by date (newest first), then by report name
        reports.sort(key=lambda x: (x['date'], x['report_name']), reverse=True)
        
        return jsonify({
            'reports': reports,
            'total_count': len(reports)
        })
        
    except Exception as e:
        return jsonify({'error': f'Error fetching reports: {str(e)}'}), 500

@reports_bp.route("/reports/<report_id>", methods=["GET"])
def get_report_signed_url(report_id):
    """
    Return the signed URL of the exact report by ID
    
    Args:
        report_id: Format "YYYY-MM-DD_report_name" (e.g., "2024-01-15_report_sales")
    
    Query Parameters:
        expiration_hours: Hours until signed URL expires (default: 1)
    
    Response format:
    {
        "id": "2024-01-15_report_sales",
        "date": "2024-01-15",
        "report_name": "report_sales",
        "filename": "report_sales.xml",
        "signed_url": "https://storage.googleapis.com/...",
        "expires_at": "2024-01-15T15:30:00Z"
    }
    """
    try:
        # Parse the report ID
        parsed_id = parse_report_id(report_id)
        if not parsed_id:
            return jsonify({
                'error': 'Invalid report ID format. Expected format: YYYY-MM-DD_report_name'
            }), 400
        
        date_str = parsed_id['date']
        report_name = parsed_id['report_name']
        
        # Construct the expected blob path
        blob_path = f"reports/{date_str}/{report_name}.xml"
        
        # Get expiration hours from query parameters
        expiration_hours = request.args.get('expiration_hours', 1, type=int)
        if expiration_hours < 1 or expiration_hours > 168:  # Max 1 week
            return jsonify({
                'error': 'expiration_hours must be between 1 and 168 (1 week)'
            }), 400
        
        bucket = get_bucket()
        blob = bucket.blob(blob_path)
        
        # Check if the blob exists
        if not blob.exists():
            return jsonify({
                'error': f'Report not found: {report_id}'
            }), 404
        
        # Generate signed URL
        expiration_time = datetime.utcnow() + timedelta(hours=expiration_hours)
        signed_url = blob.generate_signed_url(
            expiration=expiration_time,
            method='GET'
        )
        
        return jsonify({
            'id': report_id,
            'date': date_str,
            'report_name': report_name,
            'filename': f"{report_name}.xml",
            'blob_path': blob_path,
            'signed_url': signed_url,
            'expires_at': expiration_time.isoformat() + 'Z',
            'size_bytes': blob.size,
            'created': blob.time_created.isoformat() if blob.time_created else None
        })
        
    except Exception as e:
        return jsonify({'error': f'Error generating signed URL: {str(e)}'}), 500

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
