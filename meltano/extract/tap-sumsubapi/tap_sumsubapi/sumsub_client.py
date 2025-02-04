from __future__ import annotations

import hashlib
import hmac
import time
from typing import Any, Dict

import requests

REQUEST_TIMEOUT = 60


class SumsubClient:
    def __init__(self, config: Dict[str, Any]):
        self.config = config
        self.session = None

    def __enter__(self):
        """Support with-statement context management."""
        self.session = requests.Session()
        self.session.headers.update({"accept": "application/json"})
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        """Ensure the database connection is closed when exiting the context."""
        self.session.close()

    def get_applicant_data(self, external_user_id):
        """https://docs.sumsub.com/reference/get-applicant-data-via-externaluserid"""
        url = f"https://api.sumsub.com/resources/applicants/-;externalUserId={external_user_id}/one"
        resp = self.sign_request(requests.Request("GET", url))
        response = self.session.send(resp, timeout=REQUEST_TIMEOUT)
        return response

    def sign_request(self, request: requests.Request) -> requests.PreparedRequest:
        prepared_request = request.prepare()
        now = int(time.time())
        method = request.method.upper()
        path_url = prepared_request.path_url
        body = b"" if prepared_request.body is None else prepared_request.body
        if type(body) == str:
            body = body.encode("utf-8")
        data_to_sign = (
            str(now).encode("utf-8")
            + method.encode("utf-8")
            + path_url.encode("utf-8")
            + body
        )
        # hmac needs bytes
        signature = hmac.new(
            self.config.get("secret").encode("utf-8"),
            data_to_sign,
            digestmod=hashlib.sha256,
        )
        prepared_request.headers["X-App-Token"] = self.config.get("key")
        prepared_request.headers["X-App-Access-Ts"] = str(now)
        prepared_request.headers["X-App-Access-Sig"] = signature.hexdigest()
        return prepared_request
