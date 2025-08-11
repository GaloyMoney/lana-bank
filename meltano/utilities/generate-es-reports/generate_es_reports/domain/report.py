from __future__ import annotations
from typing import Any


class ReportJobDefinition:
    """
    Defines a report that must be fetched and converted into
    certain file formats.
    """

    def __init__(
        self,
        norm: str,
        id: str,
        friendly_name: str,
        file_output_configs: tuple[BaseFileOutputConfig, ...],
    ):
        self.norm = norm
        self.id = id
        self.friendly_name = friendly_name
        self.file_output_configs = file_output_configs

    @property
    def source_table_name(self) -> str:
        return f"report_{self.norm}_{self.id}"


class StorableReportOutput:
    """The contents of a report file, together with their content type."""

    def __init__(self, report_content_type: str, report_content: str) -> None:
        self.content_type = report_content_type
        self.content = report_content


class TabularReportContents:

    def __init__(self, field_names: tuple[str, ...], records: dict[str, Any]):
        self.fields = field_names
        self.records = records
