from airflow.plugins_manager import AirflowPlugin
from flask import Blueprint, jsonify, request
from google.cloud import storage
from google.oauth2 import service_account
from airflow.www.app import csrf
from airflow.models import DagRun
from airflow.utils.state import State
from airflow import settings
from airflow.api.client.local_client import Client
import logging
import os
from datetime import datetime, timedelta
import pytz

logger = logging.getLogger(__name__)

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
    
    Response format: Array<String>
    [
        "2024-01-15",
        "2024-01-14",
        "2024-01-13",
        ...
    ]
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

        return jsonify(sorted_dates)

    except Exception as e:
        return jsonify({'error': f'Error fetching available dates: {str(e)}'}), 500

@reports_bp.route("/reports/date/<date>", methods=["GET"])
def get_reports_by_date(date):
    """
    Return URIs of all reports for a given date
    
    Args:
        date: Date in YYYY-MM-DD format (e.g., "2024-01-15")
    
    Response format: Array<String>
    [
        "reports/2025-06-29/nrsf_03/funcionarios_y_empleados.txt",
        "reports/2025-06-29/nrsf_03/other_report.txt",
        ...
    ]
    """
    try:
        # Validate date format
        try:
            datetime.strptime(date, '%Y-%m-%d')
        except ValueError:
            return jsonify({
                'error': 'Invalid date format. Expected format: YYYY-MM-DD'
            }), 400

        bucket = get_bucket()

        # List all blobs for the specific date
        date_prefix = f'reports/{date}/'
        blobs = bucket.list_blobs(prefix=date_prefix)

        uris = []

        for blob in blobs:
            # Since we're using the date prefix, all blobs should be for this date
            # But we still validate the blob name format to ensure it's a valid report
            parsed = parse_report_blob(blob.name)
            if parsed:
                uris.append(blob.name)

        # Sort URIs alphabetically
        uris.sort()

        return jsonify(uris)
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

GENERATE_REPORTS_DAG_ID = 'meltano_generate-es-reports-daily_generate-es-reports-job'

def get_utc_now():
    """Get current UTC time with timezone info"""
    return datetime.utcnow().replace(tzinfo=pytz.UTC)

def ensure_dag_is_available():
    """Ensure the DAG exists and is unpaused"""
    from airflow.models import DagModel
    
    session = settings.Session()
    try:
        # Check if DAG exists in DagModel
        dag_model = session.query(DagModel).filter(
            DagModel.dag_id == GENERATE_REPORTS_DAG_ID
        ).first()
        
        if not dag_model:
            logger.error(f"DAG {GENERATE_REPORTS_DAG_ID} not found in DagModel")
            raise ValueError(f"DAG {GENERATE_REPORTS_DAG_ID} not found")
        
        # Unpause the DAG if it's paused
        if dag_model.is_paused:
            logger.info(f"Unpausing DAG {GENERATE_REPORTS_DAG_ID}")
            dag_model.is_paused = False
            session.commit()
        
        return True
        
    except Exception as e:
        session.rollback()
        logger.error(f"Error ensuring DAG availability: {str(e)}")
        raise e
    finally:
        session.close()

def get_running_dag_run():
    """Check if there's a running DAG run for generate reports"""
    session = settings.Session()
    try:
        running_dag_run = session.query(DagRun).filter(
            DagRun.dag_id == GENERATE_REPORTS_DAG_ID,
            DagRun.state.in_([State.RUNNING, State.QUEUED])  # Include QUEUED state
        ).first()
        return running_dag_run
    finally:
        session.close()

def trigger_dag_run():
    """Trigger a new DAG run for generate reports using Airflow's API"""
    try:
        # First ensure the DAG is available and unpaused
        ensure_dag_is_available()

        # Use Airflow's Local Client (recommended approach)
        client = Client(None, None)
        execution_date = get_utc_now()

        dag_run = client.trigger_dag(
            dag_id=GENERATE_REPORTS_DAG_ID,
            execution_date=execution_date,
            conf={
                "triggered_by": "reports_api",
                "trigger_time": execution_date.isoformat(),
                "api_trigger": True
            }
        )
        logger.info(f"Successfully triggered DAG {GENERATE_REPORTS_DAG_ID} with run_id: {dag_run.run_id}")
        return dag_run

    except Exception as e:
        logger.error(f"Failed to trigger DAG using Client API: {str(e)}")
        # Fallback to manual DAG run creation
        return trigger_dag_run_manual()

def trigger_dag_run_manual():
    """Fallback method: Manually create DAG run with timezone-aware datetime"""
    session = settings.Session()
    try:
        execution_date = get_utc_now()

        # Import DagRunType for newer Airflow versions
        try:
            from airflow.utils.types import DagRunType
            run_type = DagRunType.MANUAL
        except ImportError:
            # For older Airflow versions
            run_type = "manual"

        # Create a new DAG run with timezone-aware datetime
        dag_run = DagRun(
            dag_id=GENERATE_REPORTS_DAG_ID,
            execution_date=execution_date,  # Now timezone-aware
            run_id=f"api_trigger_{execution_date.strftime('%Y%m%d_%H%M%S')}",
            state=State.QUEUED,
            external_trigger=True,
            run_type=run_type,  # Add required run_type field
            conf={
                "triggered_by": "reports_api",
                "trigger_time": execution_date.isoformat(),
                "api_trigger": True
            }
        )
        session.add(dag_run)
        session.commit()

        logger.info(f"Manually created DAG run {dag_run.run_id}")
        return dag_run

    except Exception as e:
        session.rollback()
        logger.error(f"Failed to manually create DAG run: {str(e)}")
        raise e
    finally:
        session.close()

def format_datetime_for_json(dt):
    """Format datetime for JSON response with proper timezone"""
    if not dt:
        return None
    # If datetime is naive, assume UTC
    if dt.tzinfo is None:
        dt = dt.replace(tzinfo=pytz.UTC)
    # Convert to ISO format
    iso_string = dt.isoformat()
    # Ensure it ends with Z for UTC times
    if iso_string.endswith('+00:00'):
        iso_string = iso_string[:-6] + 'Z'
    elif not iso_string.endswith('Z') and '+' not in iso_string[-6:]:
        iso_string += 'Z'
    return iso_string

@reports_bp.route("/reports/generate", methods=["POST"])
@csrf.exempt
def generate_reports():
    """
    Trigger generation of ES reports via Airflow DAG

    Response format:
    - If job starts successfully: {"status": "STARTED", "dag_run_id": "api_trigger_20250628_100000", "start_time": "2025-06-28T10:00:00Z"}
    - If job is already running: {"status": "RUNNING", "dag_run_id": "existing_run_id", "start_time": "2025-06-28T09:30:00Z"}
    - If error occurs: {"status": "ERROR", "error": "error message"}
    """
    try:
        # Check if there's already a running DAG run
        running_dag_run = get_running_dag_run()
        if running_dag_run:
            return jsonify({
                'status': 'RUNNING',
                'dag_run_id': running_dag_run.run_id,
                'start_time': format_datetime_for_json(running_dag_run.start_date),
                'current_state': running_dag_run.state
            })

        # Trigger a new DAG run
        dag_run = trigger_dag_run()

        return jsonify({
            'status': 'STARTED',
            'dag_run_id': dag_run.run_id,
            'start_time': format_datetime_for_json(dag_run.execution_date)
        })

    except Exception as e:
        logger.error(f"Error in generate_reports endpoint: {str(e)}")
        return jsonify({
            'status': 'ERROR', 
            'error': str(e)
        }), 500

@reports_bp.route("/reports/generate/status", methods=["GET"])
def get_generate_status():
    """
    Get the current status of the generate reports DAG

    Response format:
    {
        "is_running": true/false,
        "current_run": {
            "dag_run_id": "run_id",
            "state": "running|success|failed|queued",
            "start_time": "2025-06-28T10:00:00Z",
            "end_time": "2025-06-28T11:00:00Z" or null
        } or null,
        "last_completed_run": {...} or null
    }
    """
    try:
        session = settings.Session()
        try:
            # Get current running/queued DAG run
            running_dag_run = session.query(DagRun).filter(
                DagRun.dag_id == GENERATE_REPORTS_DAG_ID,
                DagRun.state.in_([State.RUNNING, State.QUEUED])
            ).order_by(DagRun.execution_date.desc()).first()

            # Get last completed DAG run
            last_completed_run = session.query(DagRun).filter(
                DagRun.dag_id == GENERATE_REPORTS_DAG_ID,
                DagRun.state.in_([State.SUCCESS, State.FAILED, State.UPSTREAM_FAILED, State.SKIPPED])
            ).order_by(DagRun.execution_date.desc()).first()

            def format_dag_run_info(dag_run):
                """Helper function to format DAG run information"""
                if not dag_run:
                    return None

                return {
                    'dag_run_id': dag_run.run_id,
                    'state': dag_run.state,
                    'start_time': format_datetime_for_json(dag_run.start_date),
                    'end_time': format_datetime_for_json(dag_run.end_date),
                    'execution_date': format_datetime_for_json(dag_run.execution_date)
                }

            current_run = format_dag_run_info(running_dag_run)
            last_completed = format_dag_run_info(last_completed_run)

            return jsonify({
                'is_running': running_dag_run is not None,
                'current_run': current_run,
                'last_completed_run': last_completed,
                'dag_id': GENERATE_REPORTS_DAG_ID
            })

        finally:
            session.close()
    except Exception as e:
        logger.error(f"Error in get_generate_status endpoint: {str(e)}")
        return jsonify({
            'error': f'Error getting DAG status: {str(e)}'
        }), 500

class ReportsApiPlugin(AirflowPlugin):
    """Airflow plugin to expose reports API endpoints"""
    name = "reports_api"
    flask_blueprints = [reports_bp]
