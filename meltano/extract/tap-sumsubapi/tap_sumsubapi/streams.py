"""Stream type classes for tap-sumsubapi."""

from __future__ import annotations

import typing as t
from typing import Iterable, Dict, Any

import json
from datetime import datetime

import requests
from singer_sdk import Stream
from singer_sdk import typing as th  # JSON Schema typing helpers

from tap_sumsubapi.postgres_client import PostgresClient
from tap_sumsubapi.sumsub_client import SumsubClient


class ApplicantStream(Stream):
    name = "sumsub_applicants"
    path = "resources/applicants"
    primary_keys: t.ClassVar[list[str]] = ["customer_id", "recorded_at"]
    replication_key = "recorded_at"
    schema = th.PropertiesList(
        th.Property("customer_id", th.StringType),
        th.Property("recorded_at", th.DateTimeType),
        th.Property(
            "content",
            th.StringType,
            description="response from sumsub API",
        ),
        th.Property(
            "document_images",
            th.ArrayType(
                th.ObjectType(
                    th.Property("image_id", th.StringType),
                    th.Property("base64_image", th.StringType),
                )
            ),
            description="Base64 encoded document images",
        ),
    ).to_dict()

    def _starting_timestamp(self, context):
        return self.get_starting_replication_key_value(context) or datetime.min

    def __init__(self, tap):
        super().__init__(tap)
        self.postgres_client = PostgresClient(
            {
                "host": tap.postgres_host,
                "port": tap.postgres_port,
                "database": tap.postgres_database,
                "user": tap.postgres_user,
                "password": tap.postgres_password,
                "sslmode": tap.config.get("sslmode", "prefer"),
            }
        )
        self.sumsub_client = SumsubClient(
            {
                "key": tap.sumsub_key,
                "secret": tap.sumsub_secret,
            }
        )

    def get_records(self, context: Dict[str, Any]) -> Iterable[Dict[str, Any]]:
        """Generator function that yields records."""
        with self.postgres_client as pg_client:
            keys = pg_client.get_keys(
                starting_timestamp=self._starting_timestamp(context),
            )
            with self.sumsub_client as ss_client:
                for customer_id, recorded_at in keys:
                    try:
                        response = ss_client.get_applicant_data(customer_id)
                        content = response.text
                        response_json = response.json()
                        document_images = []
                        if "id" in response_json:
                            metadata = ss_client.get_document_metadata(
                                response_json["id"]
                            )
                            for item in metadata.get("items", []):
                                image_id = item.get("id")
                                inspection_id = response_json["inspectionId"]
                                base64_image = ss_client.download_document_image(
                                    inspection_id, image_id
                                )
                                document_images.append(
                                    {
                                        "image_id": image_id,
                                        "base64_image": base64_image,
                                    }
                                )
                    except requests.exceptions.RequestException as e:
                        content = json.dumps({"error": e})
                        document_images = []
                    yield {
                        "customer_id": customer_id,
                        "recorded_at": recorded_at,
                        "content": content,
                        "document_images": document_images,
                    }
