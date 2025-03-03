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
                SELECT customer_id, recorded_at
                FROM sumsub_callbacks
                WHERE recorded_at > %s
            """
            cursor.execute(query, (starting_timestamp,))
            yield from cursor
