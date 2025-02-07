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
    primary_keys: t.ClassVar[list[str]] = ["customer_id"]
    replication_key = "recorded_at"
    schema = th.PropertiesList(
        th.Property("customer_id", th.StringType),
        th.Property("recorded_at", th.DateTimeType),
        th.Property(
            "content",
            th.StringType,
            description="response from sumsub API",
        ),
    ).to_dict()

    def _starting_timestamp(self, context):
        return self.get_starting_replication_key_value(context) or datetime.min

    def __init__(self, tap):
        super().__init__(tap)
        self.postgres_client = PostgresClient(
            {
                "host": tap.config["host"],
                "port": tap.config.get("port", 5432),
                "database": tap.config["database"],
                "user": tap.config["user"],
                "password": tap.config["password"],
                "sslmode": tap.config.get("sslmode", "prefer"),
            }
        )
        self.sumsub_client = SumsubClient(
            {
                "key": tap.config["key"],
                "secret": tap.config["secret"],
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
                    except requests.exceptions.RequestException as e:
                        content = json.dumps({"error": e})
                    yield {
                        "customer_id": customer_id,
                        "recorded_at": recorded_at,
                        "content": content,
                    }
