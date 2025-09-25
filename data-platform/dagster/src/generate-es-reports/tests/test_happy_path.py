from xmlschema import XMLSchema


from generate_es_reports.domain.report import (
    ReportBatch,
    ReportJobDefinition,
    TXTFileOutputConfig,
    CSVFileOutputConfig,
    XMLFileOutputConfig,
)
from generate_es_reports.io import MockTableFetcher, MockTable, LocalReportStorer


def test_write_some_report_produces_semantically_matching_outputs(
    tmp_path,
):  # tmp_path is a pytest managed dir that cleans up automagically

    xsd_string = """<?xml version="1.0" encoding="UTF-8"?>
        <xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
                targetNamespace="http://example.com/silly/pizza"
                xmlns="http://example.com/silly/pizza"
                elementFormDefault="qualified">

            <xs:element name="pizza_orders">
                <xs:complexType>
                    <xs:sequence>
                        <xs:element name="order" maxOccurs="unbounded">
                            <xs:complexType>
                                <xs:sequence>
                                    <xs:element name="order_id" type="xs:integer"/>
                                    <xs:element name="pizza" type="xs:string"/>
                                    <xs:element name="quantity" type="xs:integer"/>
                                    <xs:element name="created_at" type="xs:dateTime"/>
                                </xs:sequence>
                            </xs:complexType>
                        </xs:element>
                    </xs:sequence>
                </xs:complexType>
            </xs:element>

        </xs:schema>
        """

    schema = XMLSchema(xsd_string)

    report_job_definition = ReportJobDefinition(
        norm="pizza_law",
        id="pizza_orders",
        friendly_name="pizza_orders",
        file_output_configs=(
            TXTFileOutputConfig(),
            CSVFileOutputConfig(),
            XMLFileOutputConfig(xml_schema=schema),
        ),
    )

    table_fetcher = MockTableFetcher()
    table_fetcher.add_mock_table(
        mock_table=MockTable(
            name="report_pizza_law_pizza_orders",
            records=(
                {
                    "order_id": "1",
                    "pizza": "pepperoni",
                    "quantity": "5",
                    "created_at": "2025-01-01T00:00:00Z",
                },
                {
                    "order_id": "2",
                    "pizza": "marinara",
                    "quantity": "1",
                    "created_at": "2025-01-01T00:05:00Z",
                },
                {
                    "order_id": "3",
                    "pizza": "diavola",
                    "quantity": "99",
                    "created_at": "2025-01-01T00:10:00Z",
                },
            ),
        )
    )
    report_storer = LocalReportStorer(root_path=tmp_path)

    report_batch = ReportBatch(
        run_id="test_run",
        report_jobs=(report_job_definition,),
        table_fetcher=table_fetcher,
        report_storer=report_storer,
    )

    report_batch.generate_batch()

    from_txt_str = open(
        tmp_path / "reports" / "test_run" / "pizza_law" / "pizza_orders.txt"
    ).read()
    from_csv_str = open(
        tmp_path / "reports" / "test_run" / "pizza_law" / "pizza_orders.csv"
    ).read()

    # If code runs fine until here, and the TXT and CSV versions of the output
    # are the same, we kind of assume all went well.
    assert from_csv_str == from_txt_str.replace("|", ",")
