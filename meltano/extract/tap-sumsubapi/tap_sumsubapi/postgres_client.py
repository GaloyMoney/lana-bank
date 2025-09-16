from __future__ import annotations

from typing import Any, Dict

import psycopg2


class PostgresClient:
    def __init__(self, config: Dict[str, Any]):
        self.config = config
        self.connection = None

    def __enter__(self):
        """Establish a connection to the PostgreSQL database."""
        self.connection = psycopg2.connect(
            host=self.config.get("host"),
            port=self.config.get("port", 5432),
            dbname=self.config.get("database"),
            user=self.config.get("user"),
            password=self.config.get("password"),
            sslmode=self.config.get("sslmode", "prefer"),
        )
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        """Close the connection to the PostgreSQL database."""
        if self.connection:
            self.connection.close()
            self.connection = None

    def get_keys(self, starting_timestamp):
        with self.connection.cursor() as cursor:
            query = """
                with customer_ids as (
                    select distinct customer_id
                    from sumsub_callbacks
                    where recorded_at > %s
                    and content->>'type' in ('applicantReviewed', 'applicantPersonalInfoChanged')
                )
                select customer_id, now()
                from customer_ids
            """
            cursor.execute(query, (starting_timestamp,))
            yield from cursor
