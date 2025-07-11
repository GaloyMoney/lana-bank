import os, logging, pytz
from datetime import datetime
from flask import Blueprint, jsonify
from google.cloud import storage
from google.oauth2 import service_account
from airflow.plugins_manager import AirflowPlugin
from airflow.www.app import csrf
from airflow import settings
from airflow.models import DagRun, TaskInstance
from airflow.utils.state import State
from airflow.api.client.local_client import Client
from airflow.utils.log.log_reader import TaskLogReader

logger = logging.getLogger(__name__)
reports_bp = Blueprint("reports_api", __name__, url_prefix="/api/v1")

DAG_ID = "meltano_generate-es-reports-daily_generate-es-reports-job"

# ──────────────── GCS helpers ────────────────
def _storage():
    creds = service_account.Credentials.from_service_account_file(
        os.environ["GOOGLE_APPLICATION_CREDENTIALS"]
    )
    return storage.Client(project=os.getenv("DBT_BIGQUERY_PROJECT"), credentials=creds)

def _bucket():
    return _storage().bucket(os.environ["DOCS_BUCKET_NAME"])

def _parse(blob):
    parts = blob.split("/")
    if len(parts) == 4 and parts[0] == "reports":
        try:
            datetime.strptime(parts[1], "%Y-%m-%d")
            return parts[1]
        except ValueError:
            pass

# ──────────────── /reports/dates ────────────────
@reports_bp.route("/reports/dates")
def dates():
    try:
        blobs = _bucket().list_blobs(prefix="reports/")
        dates = {_parse(b.name) for b in blobs if _parse(b.name)}
        return jsonify(sorted(dates, reverse=True))
    except Exception as e:
        return jsonify(error=str(e)), 500

# ──────────────── /reports/date/<date> ────────────────
@reports_bp.route("/reports/date/<date>")
def by_date(date):
    try:
        datetime.strptime(date, "%Y-%m-%d")
        blobs = _bucket().list_blobs(prefix=f"reports/{date}/")
        uris = sorted([b.name for b in blobs if _parse(b.name)])
        return jsonify(uris)
    except ValueError:
        return jsonify(error="Bad date"), 400
    except Exception as e:
        return jsonify(error=str(e)), 500

# ──────────────── /reports/health ────────────────
@reports_bp.route("/reports/health")
def health():
    try:
        _bucket().exists()
        return jsonify(status="healthy")
    except Exception as e:
        return jsonify(status="unhealthy", error=str(e)), 500


# ──────────────── /reports/generate ────────────────
@reports_bp.route("/reports/generate", methods=["POST"])
@csrf.exempt
def generate():
    try:
        dr = _running()
        if dr:
            return jsonify(run_id=dr.run_id)
        run_id = f"api__{_utc().isoformat()}"
        Client(None, None).trigger_dag(
            DAG_ID,
            run_id=run_id,
            execution_date=_utc(),
            conf={"triggered_by": "reports_api"}
        )
        return jsonify(run_id=run_id)
    except Exception as e:
        logger.error(e)
        return jsonify(error=str(e)), 500

# ──────────────── TaskLogReader and Status helpers ────────────────
_reader = TaskLogReader()

def _utc():
    return datetime.utcnow().replace(tzinfo=pytz.UTC)

def _run_type(dr):
    """scheduled | api_triggered"""
    if dr.external_trigger and dr.conf and dr.conf.get("api_trigger"):
        return "api_triggered"
    return "scheduled"

def _running():
    s = settings.Session()
    try:
        return s.query(DagRun).filter(
            DagRun.dag_id == DAG_ID,
            DagRun.state.in_([State.RUNNING, State.QUEUED])
        ).first()
    finally:
        s.close()

def _collect_logs(session, run_id):
    tis = (
        session.query(TaskInstance)
        .filter(TaskInstance.dag_id == DAG_ID, TaskInstance.run_id == run_id)
        .order_by(TaskInstance.task_id.asc())
        .all()
    )

    def flatten(obj):
        if isinstance(obj, str):
            yield obj
        elif hasattr(obj, "message"):
            yield obj.message
        elif isinstance(obj, (list, tuple)):
            for x in obj:
                yield from flatten(x)
        else:
            yield str(obj)

    pieces = []
    for ti in tis:
        chunks, _ = _reader.read_log_chunks(ti, ti.try_number, metadata={})
        pieces.append(f"\n\n===== {ti.task_id} =====\n")
        pieces.append("".join(flatten(chunks)))

    return "".join(pieces) if pieces else ""

# ──────────────── /reports/status ────────────────
@reports_bp.route("/reports/status")
def status():
    """Return running status, logs, and info on the most recent finished run."""
    try:
        session = settings.Session()

        # ── current run (RUNNING or QUEUED) ───────────────────────────────────
        current = (
            session.query(DagRun)
            .filter(
                DagRun.dag_id == DAG_ID,
                DagRun.state.in_([State.RUNNING, State.QUEUED]),
            )
            .order_by(DagRun.execution_date.desc())
            .first()
        )

        running = bool(current)
        run_type = run_started_at = logs = None
        if running:
            run_type = _run_type(current)
            run_started_at = current.start_date.isoformat() if current.start_date else None
            logs = _collect_logs(session, current.run_id)

        # ── last *finished* run ───────────────────────────────────────────────
        last = (
            session.query(DagRun)
            .filter(
                DagRun.dag_id == DAG_ID,
                DagRun.state.in_([State.SUCCESS, State.FAILED]),
            )
            .order_by(DagRun.execution_date.desc())
            .first()
        )

        last_run = None
        if last:
            last_run = {
                "run_type": _run_type(last),
                "run_started_at": last.start_date.isoformat() if last.start_date else None,
                "status": "success" if last.state == State.SUCCESS else "failed",
                "logs": _collect_logs(session, last.run_id),
            }

        return jsonify(
            running=running,
            run_type=run_type,
            run_started_at=run_started_at,
            logs=logs,
            last_run=last_run,
        )
    except Exception as e:
        logger.error("status endpoint error: %s", e)
        return jsonify(
            running=False,
            run_type=None,
            run_started_at=None,
            logs=None,
            last_run=None,
            error=str(e),
        ), 500
    finally:
        session.close()

class ReportsApiPlugin(AirflowPlugin):
    name = "reports_api"
    flask_blueprints = [reports_bp]
