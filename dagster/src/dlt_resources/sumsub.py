from __future__ import annotations

import base64
import hashlib
import hmac
import logging
import time
from datetime import datetime, timezone
from typing import Any, Dict, Iterator, List, Optional, Tuple

import dlt
import psycopg2
from dlt.sources.helpers import requests

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


def _get_customers(conn_str: str, since: datetime) -> List[Tuple[str, datetime]]:
    """Return (customer_id, max_recorded_at) for callbacks strictly after 'since', ordered by max_recorded_at."""
    with psycopg2.connect(conn_str) as conn:
        with conn.cursor() as cursor:
            cursor.execute(
                """
                select customer_id, max(recorded_at) as max_recorded_at
                from sumsub_callbacks
                where recorded_at > %s
                group by customer_id
                order by max(recorded_at) asc
                """,
                (since,),
            )
            return [(row[0], row[1]) for row in cursor]


@dlt.resource(
    name="sumsub_applicants_dlt",
    write_disposition="append",
    primary_key=["customer_id", "recorded_at"],
)
def applicants(
    pg_connection_string: str,
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
    start_ts: datetime = callbacks_since.last_value or datetime(
        1970, 1, 1, tzinfo=timezone.utc
    )
    LOGGER.info("Starting Sumsub applicants sync from %s", start_ts)

    with requests.Session() as session:
        customer_rows: List[Tuple[str, datetime]] = _get_customers(
            pg_connection_string, start_ts
        )
        for customer_id, max_recorded_at in customer_rows:
            LOGGER.info(
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
                LOGGER.warning(
                    "Applicant fetch failed for customer_id=%s (will retry next run): %s",
                    customer_id,
                    e,
                )
                # Stop processing to avoid advancing the incremental cursor past the failure.
                break

            try:
                resp_json = resp.json()
            except ValueError as e:
                LOGGER.warning(
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
                    LOGGER.warning(
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
                            LOGGER.warning(
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
