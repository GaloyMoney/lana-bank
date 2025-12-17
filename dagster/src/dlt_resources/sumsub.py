from __future__ import annotations

import base64
import hashlib
import hmac
import logging
import time
from datetime import datetime, timezone
from typing import Any, Dict, Iterator, List, Optional, Tuple

import dlt
import dagster as dg
from dlt.sources.helpers import requests
from google.cloud import bigquery
from google.oauth2 import service_account

LOGGER = logging.getLogger(__name__)

REQUEST_TIMEOUT = 60
SUMSUB_API_BASE = "https://api.sumsub.com"


def _sumsub_send(
    session: requests.Session,
    method: str,
    url: str,
    key: str,
    secret: str,
    body: Optional[bytes] = None,
) -> requests.Response:
    """Prepare and send a signed Sumsub API request."""
    req = requests.Request(method, url, data=body)
    prepared = session.prepare_request(req)

    now_ts = int(time.time())
    method_upper = method.upper()
    path_url = prepared.path_url
    body_bytes = b"" if prepared.body is None else prepared.body
    if isinstance(body_bytes, str):
        body_bytes = body_bytes.encode("utf-8")

    data_to_sign = (
        str(now_ts).encode("utf-8")
        + method_upper.encode("utf-8")
        + path_url.encode("utf-8")
        + body_bytes
    )
    signature = hmac.new(secret.encode("utf-8"), data_to_sign, hashlib.sha256)

    prepared.headers["accept"] = "application/json"
    prepared.headers["X-App-Token"] = key
    prepared.headers["X-App-Access-Ts"] = str(now_ts)
    prepared.headers["X-App-Access-Sig"] = signature.hexdigest()

    return session.send(prepared, timeout=REQUEST_TIMEOUT)


def _get_applicant_data(
    session: requests.Session, external_user_id: str, key: str, secret: str
) -> requests.Response:
    url = f"{SUMSUB_API_BASE}/resources/applicants/-;externalUserId={external_user_id}/one"
    return _sumsub_send(session, "GET", url, key, secret)


def _get_document_metadata(
    session: requests.Session, applicant_id: str, key: str, secret: str
) -> Dict[str, Any]:
    url = f"{SUMSUB_API_BASE}/resources/applicants/{applicant_id}/metadata/resources"
    resp = _sumsub_send(session, "GET", url, key, secret)
    resp.raise_for_status()
    return resp.json()


def _download_document_image(
    session: requests.Session, inspection_id: str, image_id: str, key: str, secret: str
) -> Optional[str]:
    url = (
        f"{SUMSUB_API_BASE}/resources/inspections/{inspection_id}/resources/{image_id}"
    )
    resp = _sumsub_send(session, "GET", url, key, secret)
    if resp.status_code == 200:
        return base64.b64encode(resp.content).decode("utf-8")
    return None


def _get_bq_client(credentials_dict: Dict[str, Any]) -> bigquery.Client:
    """Create a BigQuery client from service account credentials dict."""
    creds = service_account.Credentials.from_service_account_info(credentials_dict)
    project_id = credentials_dict["project_id"]
    return bigquery.Client(project=project_id, credentials=creds)


def _get_customers_bq(
    credentials_dict: Dict[str, Any], dataset: str, since: datetime
) -> List[Tuple[str, datetime]]:
    """Return (customer_id, max_recorded_at) for callbacks on/after 'since', ordered by max_recorded_at."""
    client = _get_bq_client(credentials_dict)
    table = f"`{credentials_dict['project_id']}.{dataset}.sumsub_callbacks`"
    sql = f"""
      WITH customers AS (
        SELECT customer_id, MAX(recorded_at) AS recorded_at
        FROM {table}
        WHERE recorded_at >= @since
        GROUP BY customer_id
      )
      SELECT customer_id, recorded_at
      FROM customers
      ORDER BY recorded_at ASC
    """
    job_config = bigquery.QueryJobConfig(
        query_parameters=[bigquery.ScalarQueryParameter("since", "TIMESTAMP", since)]
    )
    rows = list(client.query(sql, job_config=job_config))
    return [(row["customer_id"], row["recorded_at"]) for row in rows]


@dlt.resource(
    name="sumsub_applicants_dlt",
    write_disposition="append",
    primary_key=["customer_id", "recorded_at"],
)
def applicants(
    bq_credentials: Dict[str, Any],
    bq_dataset: str,
    sumsub_key: str,
    sumsub_secret: str,
    callbacks_since=dlt.sources.incremental(
        "recorded_at", initial_value=datetime(1970, 1, 1, tzinfo=timezone.utc)
    ),
) -> Iterator[Dict[str, Any]]:
    """
    Fetch applicant data from Sumsub for customers with callbacks since the last run.

    - One row per customer, using the maximum recorded_at from sumsub_callbacks as the incremental cursor.
    - Do not emit a row on applicant fetch/JSON failure; stop processing to retry on the next run.
    - Metadata/image fetch failures are non-fatal and only affect document_images.
    """
    logger = dg.get_dagster_logger()
    start_ts: datetime = callbacks_since.last_value or datetime(
        1970, 1, 1, tzinfo=timezone.utc
    )
    logger.info("Starting Sumsub applicants sync from %s", start_ts)

    with requests.Session() as session:
        customer_rows: List[Tuple[str, datetime]] = _get_customers_bq(
            bq_credentials, bq_dataset, start_ts
        )
        for customer_id, max_recorded_at in customer_rows:
            logger.info(
                "Fetching Sumsub data for customer_id=%s recorded_at=%s",
                customer_id,
                max_recorded_at,
            )

            # Narrow try: only wrap network/HTTP and JSON parsing for the main applicant call.
            try:
                resp = _get_applicant_data(
                    session, customer_id, sumsub_key, sumsub_secret
                )
                resp.raise_for_status()
            except requests.exceptions.RequestException as e:
                logger.warning(
                    "Applicant fetch failed for customer_id=%s (will retry next run): %s",
                    customer_id,
                    e,
                )
                # Stop processing to avoid advancing the incremental cursor past the failure.
                break

            try:
                resp_json = resp.json()
            except ValueError as e:
                logger.warning(
                    "Invalid JSON from Sumsub for customer_id=%s (will retry next run): %s",
                    customer_id,
                    e,
                )
                # Stop processing to avoid advancing the incremental cursor past the failure.
                break

            content_text = resp.text
            document_images: List[Dict[str, Optional[str]]] = []

            applicant_id = resp_json.get("id")
            inspection_id = resp_json.get("inspectionId")

            if applicant_id:
                try:
                    metadata = _get_document_metadata(
                        session, applicant_id, sumsub_key, sumsub_secret
                    )
                except requests.exceptions.RequestException as e:
                    logger.warning(
                        "Metadata fetch failed for customer_id=%s (continuing without images): %s",
                        customer_id,
                        e,
                    )
                    metadata = {"items": []}

                for item in metadata.get("items", []):
                    image_id = item.get("id")
                    if image_id and inspection_id:
                        try:
                            base64_image = _download_document_image(
                                session,
                                inspection_id,
                                image_id,
                                sumsub_key,
                                sumsub_secret,
                            )
                        except requests.exceptions.RequestException as e:
                            logger.warning(
                                "Image download failed for customer_id=%s image_id=%s: %s",
                                customer_id,
                                image_id,
                                e,
                            )
                            base64_image = None
                        document_images.append(
                            {"image_id": image_id, "base64_image": base64_image}
                        )

            yield {
                "customer_id": customer_id,
                "recorded_at": max_recorded_at,
                "content": content_text,
                "document_images": document_images,
            }
