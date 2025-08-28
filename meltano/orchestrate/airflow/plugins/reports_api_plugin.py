"""
Reports API plugin for Airflow.

Exposes:

  GET  /api/v1/reports/health
  GET  /api/v1/reports?limit=100&after=<run_id>
  GET  /api/v1/report/<run_id>
  POST /api/v1/reports/generate
"""
from __future__ import annotations

import os, logging
from datetime import datetime, timezone, timedelta
from typing import Protocol, TypedDict, Optional, Sequence

from flask import Blueprint, jsonify, request
from airflow.plugins_manager import AirflowPlugin
from airflow.www.app import csrf
from airflow import settings
from airflow.api.client.local_client import Client
from airflow.models import DagRun
from airflow.utils.session import provide_session
from google.cloud import storage
from google.oauth2 import service_account
from sqlalchemy.orm import Session
import re
import base64

logger = logging.getLogger(__name__)
DAG_ID = "meltano_generate-es-reports-daily_generate-es-reports-job"
REPORT_PREFIX = "reports/"

class Run(TypedDict):
    run_id: str
    execution_date: str
    state: str
    run_type: str
    start_date: Optional[str]
    end_date: Optional[str]

class File(TypedDict):
    extension: str
    path_in_bucket: str

class Report(TypedDict):
    id: str
    name: str
    norm: str
    files: Sequence[File]


class StoragePort(Protocol):
    def healthy(self) -> bool: ...
    def list_reports_for_run(self, run_id: str) -> Sequence[Report]: ...

class AirflowPort(Protocol):
    def healthy(self) -> bool: ...
    def list_runs(self, limit: int, after: str | None = None) -> Sequence[Run]: ...
    def get_run(self, run_id: str) -> Run | None: ...
    def trigger_run(self, run_id: str) -> str: ...

class ReportService:
    """Pure-logic façade that the Flask layer calls."""

    def __init__(self, airflow: AirflowPort, storage: StoragePort) -> None:
        self.airflow, self.storage = airflow, storage

    def health(self) -> dict:
        return {
            "airflow": "healthy" if self.airflow.healthy() else "unhealthy",
            "storage": "healthy" if self.storage.healthy() else "unhealthy",
        }

    def list_runs(self, limit: int, after: str | None) -> Sequence[Run]:
        return self.airflow.list_runs(limit, after)

    def get_run_details(self, run_id: str) -> Optional[dict]:
        run = self.airflow.get_run(run_id)
        if run:
            run["reports"] = self.storage.list_reports_for_run(run_id)
        return run

    def trigger_report_generation(self) -> str:
        ts = (datetime.now(timezone.utc) - timedelta(days=1)).isoformat()
        run_id = f"manual__{ts}"
        return self.airflow.trigger_run(run_id)

class AirflowAdapter(AirflowPort):

    @staticmethod
    def healthy() -> bool:
        try:
            s = settings.Session()
            s.execute("SELECT 1")
            s.close()
            return True
        except Exception:
            return False

    @provide_session
    def list_runs(
        self, limit: int, after: str | None = None, session: Session | None = None
    ) -> Sequence[Run]:
        dag_runs = (
            session.query(DagRun)
            .filter(DagRun.dag_id == DAG_ID)
            .all()
        )

        def _run_id_to_dt(run_id: str) -> datetime:
            _RUN_ID_RE = re.compile(r"^(?:manual|scheduled)__(?P<ts>.+)$")
            m = _RUN_ID_RE.match(run_id)
            if not m:
                raise ValueError(f"Unrecognised run_id format: {run_id}")

            ts = m["ts"].strip()

            # If “+” was swallowed by URL decoding, fix it.
            if " " in ts and not ts.endswith("Z"):
                ts = ts.replace(" ", "+", 1)

            # If the string still lacks an offset, assume UTC.
            if re.fullmatch(r".*\d{2}:\d{2}$", ts) and "+" not in ts and "-" not in ts:
                ts += "+00:00"

            return datetime.fromisoformat(ts)

        mapped: list[tuple[datetime, Run]] = [
            (_run_id_to_dt(dr.run_id), self._to_run(dr)) for dr in dag_runs
        ]
        mapped.sort(key=lambda pair: pair[0])

        if after:
            after_dt = _run_id_to_dt(after)
            mapped = [t for t in mapped if t[0] > after_dt]

        return [run for _, run in mapped[:limit]]

    @provide_session
    def get_run(
        self, run_id: str, session: Session | None = None
    ) -> Optional[Run]:
        dr = (
            session.query(DagRun)
            .filter(DagRun.dag_id == DAG_ID, DagRun.run_id == run_id)
            .first()
        )
        return self._to_run(dr) if dr else None

    def trigger_run(self, run_id: str) -> str:
        Client(None, None).trigger_dag(
            DAG_ID, run_id=run_id, execution_date=datetime.now(timezone.utc)
        )
        return run_id


    @staticmethod
    def _to_run(dr: DagRun) -> Run:
        return {
            "run_id": dr.run_id,
            "execution_date": dr.execution_date.isoformat(),
            "state": dr.state,
            "run_type": dr.run_type,
            "start_date": dr.start_date.isoformat() if dr.start_date else None,
            "end_date": dr.end_date.isoformat() if dr.end_date else None,
        }


class GCSAdapter(StoragePort):
    # reports/{run_id}/{norm_name}/{report_name}.{extension}
    REPORT_RE = re.compile(
        rf"^{re.escape(REPORT_PREFIX)}"
        r"(?P<run_id>[^/]+)/"                    # run-id
        r"(?P<norm>[0-9a-z_]+)/"                 # norm
        r"(?P<name>[0-9a-z_]+)\.(?P<ext>[a-z]+)$",
        re.IGNORECASE,
    )

    def __init__(self) -> None:
        creds = service_account.Credentials.from_service_account_file(
            os.environ["GOOGLE_APPLICATION_CREDENTIALS"]
        )
        self.client = storage.Client(
            project=os.getenv("DBT_BIGQUERY_PROJECT"), credentials=creds
        )
        self.bucket = self.client.bucket(os.environ["DOCS_BUCKET_NAME"])

    def healthy(self) -> bool:
        try:
            return self.bucket.exists()
        except Exception:
            return False

    def list_reports_for_run(self, run_id: str) -> Sequence[Report]:
        prefix, ts = run_id.split("__", 1)
        ts_encoded = base64.urlsafe_b64encode(ts.encode("utf-8")).decode("ascii").rstrip("=")
        run_id_encoded = f"{prefix}__{ts_encoded}"
        blobs = self.bucket.list_blobs(prefix=f"{REPORT_PREFIX}{run_id_encoded}/")

        grouped: dict[tuple[str, str], list[File]] = {}
        for blob in blobs:
            m = self.REPORT_RE.match(blob.name)
            if not m:
                continue
            if m.group("run_id") != run_id_encoded:
                continue

            key = (m.group("norm"), m.group("name"))
            grouped.setdefault(key, []).append(
                {
                    "extension": m.group("ext"),
                    "path_in_bucket": blob.name,
                }
            )

        return [
            {"norm": norm, "name": name, "files": files, "id": f"{run_id}/{norm}/{name}"}
            for (norm, name), files in grouped.items()
        ]

# ───────────────────────── Flask API ─────────────────────────
svc = ReportService(airflow=AirflowAdapter(), storage=GCSAdapter())
bp  = Blueprint("reports_api", __name__, url_prefix="/api/v1")

@bp.route("/reports/health")
def health():
    airflow = svc.airflow.healthy()
    storage = svc.storage.healthy()
    if airflow and storage:
        return jsonify(status="healthy")
    return jsonify(status="unhealthy", airflow=airflow, storage=storage), 503

@bp.route("/reports/")
def list_runs():
    limit = int(request.args.get("limit", 100))
    after = request.args.get("after")
    return jsonify(svc.list_runs(limit, after))

@bp.route("/reports/<run_id>/")
def run_details(run_id: str):
    run = svc.get_run_details(run_id)
    return (jsonify(run), 200) if run else (jsonify(error="not found"), 404)

@bp.route("/reports/generate/", methods=["POST"])
@csrf.exempt
def generate():
    try:
        return jsonify(run_id=svc.trigger_report_generation())
    except Exception as exc:
        logger.error("trigger error: %s", exc, exc_info=True)
        return jsonify(error=str(exc)), 500

class ReportsApiPlugin(AirflowPlugin):
    name = "reports_api"
    flask_blueprints = [bp]
