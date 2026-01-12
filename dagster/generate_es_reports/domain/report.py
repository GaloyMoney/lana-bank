from __future__ import annotations
from abc import ABC, abstractmethod
from typing import Any
from xml.etree import ElementTree
import io
import csv
from typing import Union

from xmlschema import XMLSchema


class StorableReportOutput:
    """The contents of a report file, together with their content type."""

    def __init__(self, report_content_type: str, report_content: str) -> None:
        self.content_type = report_content_type
        self.content = report_content


class TabularReportContents:
    """
    The data that comes from a table and will be formatted as different file
    formats.
    """

    def __init__(self, field_names: tuple[str, ...], records: list[dict[str, Any]]):
        self.fields = field_names
        self.records = records


class BaseFileOutputConfig(ABC):

    file_extension: str = NotImplemented
    content_type: str = NotImplemented

    def __init_subclass__(cls):

        mandatory_class_attributes = ("file_extension", "content_type")

        for attribute in mandatory_class_attributes:
            if getattr(cls, attribute) is NotImplemented:
                raise NotImplementedError(f"{cls.__name__} must define '{attribute}'")

    @abstractmethod
    def rows_to_report_output(
        self, table_contents: TabularReportContents
    ) -> StorableReportOutput:
        pass


class XMLFileOutputConfig(BaseFileOutputConfig):

    file_extension = "xml"
    content_type = "text/xml"

    def __init__(self, xml_schema: Union[XMLSchema, None] = None) -> None:
        self.xml_schema = xml_schema
        self.target_namespace = self.xml_schema.target_namespace
        self.root_element_tag = next(iter(self.xml_schema.elements), None)
        self.sequence_elements_tag = self._extract_sequence_elements_tag()

    def rows_to_report_output(
        self, table_contents: TabularReportContents
    ) -> StorableReportOutput:

        xml_root_element = ElementTree.Element(
            f"{{{self.target_namespace}}}" + f"{self.root_element_tag}"
        )

        for row in table_contents.records:
            sequence_level_element = ElementTree.SubElement(
                xml_root_element,
                f"{{{self.target_namespace}}}" + f"{self.sequence_elements_tag}",
            )
            for field, value in row.items():

                new_field_element = ElementTree.SubElement(
                    sequence_level_element,
                    f"{{{self.target_namespace}}}" + f"{field}",
                )
                new_field_element.text = value

        xml_string = ElementTree.tostring(xml_root_element, encoding="unicode")

        output = io.StringIO()
        output.write(xml_string)
        report_content = output.getvalue()

        report_has_content = len(table_contents.records) > 0
        is_xml_valid = self.xml_schema.is_valid(source=report_content)
        if report_has_content and not is_xml_valid:
            print(f"WARNING: Schema validation for report failed. Listing errors.")
            for err in self.xml_schema.iter_errors(report_content):
                print(f"  Path: {err.path}, Reason: {err.reason}")
                print(f"  Source: {err.source}")

        return StorableReportOutput(
            report_content=report_content, report_content_type=self.content_type
        )

    def _extract_sequence_elements_tag(self) -> str:
        """Extract the tag of the sequence elements of the schema.

        This makes a strong assumption that the XSD follows the common
        structure of SSF reports: one root element followed by a sequence
        of children elements, all within the same namespace.

        This will 100% break on XSD that follow other patterns.

        Returns:
            str: the tag for the sequence elements of this XSD.
        """
        elem = self.xml_schema.elements[self.root_element_tag]
        model = elem.type.content

        first_child = next(model.iter_elements(), None)

        # Strip namespace if present
        qname = first_child.name
        child_name = qname.split("}", 1)[-1] if qname.startswith("{") else qname

        return child_name


class CSVFileOutputConfig(BaseFileOutputConfig):

    file_extension = "csv"
    content_type = "text/csv"

    def __init__(self, delimiter: str = ",", lineterminator: str = "\n") -> None:
        self.delimiter = delimiter
        self.lineterminator = lineterminator

    def rows_to_report_output(
        self, table_contents: TabularReportContents
    ) -> StorableReportOutput:
        output = io.StringIO()

        writer = csv.DictWriter(
            output,
            fieldnames=table_contents.fields,
            delimiter=self.delimiter,
            lineterminator=self.lineterminator,
        )
        writer.writeheader()
        writer.writerows(table_contents.records)
        report_content = output.getvalue()

        return StorableReportOutput(
            report_content=report_content, report_content_type=self.content_type
        )


class TXTFileOutputConfig(BaseFileOutputConfig):

    file_extension = "txt"
    content_type = "text/plain"

    def __init__(self, delimiter: str = "|", lineterminator: str = "\n") -> None:
        self.delimiter = delimiter
        self.lineterminator = lineterminator

    def rows_to_report_output(
        self, table_contents: TabularReportContents
    ) -> StorableReportOutput:
        output = io.StringIO()

        writer = csv.DictWriter(
            output,
            fieldnames=table_contents.fields,
            delimiter=self.delimiter,
            lineterminator=self.lineterminator,
        )
        writer.writeheader()
        writer.writerows(table_contents.records)
        report_content = output.getvalue()

        return StorableReportOutput(
            report_content=report_content, report_content_type=self.content_type
        )


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
