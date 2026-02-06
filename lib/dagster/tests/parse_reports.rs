use dagster::graphql_client::Report;

fn parse_reports_from_logs_response(response_data: &serde_json::Value) -> Vec<Report> {
    let mut reports = Vec::new();
    let empty = vec![];

    let events = response_data["data"]["logsForRun"]["events"]
        .as_array()
        .unwrap_or(&empty);

    for event in events {
        if event["__typename"] == "MaterializationEvent" {
            let entries = event["metadataEntries"].as_array().unwrap_or(&empty);
            for entry in entries {
                let is_report_entry = entry["label"].as_str() == Some("report")
                    && entry["__typename"] == "JsonMetadataEntry";

                if let Some(json_string) = entry["jsonString"].as_str().filter(|_| is_report_entry)
                    && let Ok(parsed) = serde_json::from_str::<Report>(json_string)
                {
                    reports.push(parsed);
                }
            }
        }
    }

    reports
}

#[test]
fn test_parse_reports_from_logs_for_run_response() {
    let json_data = get_test_logs_for_run_response();
    let response: serde_json::Value = serde_json::from_str(json_data).unwrap();
    let reports = parse_reports_from_logs_response(&response);

    // Each MaterializationEvent produces one report entry (with one file each)
    // The dagster parser returns 48 entries (one per file)
    // The core-report sync job aggregates these by (norm, name) into 24 reports
    assert_eq!(reports.len(), 48, "Expected 48 report entries to be parsed");

    let nrp_41_reports: Vec<_> = reports.iter().filter(|r| r.norm == "nrp_41").collect();
    let nrp_51_reports: Vec<_> = reports.iter().filter(|r| r.norm == "nrp_51").collect();
    let nrsf_03_reports: Vec<_> = reports.iter().filter(|r| r.norm == "nrsf_03").collect();

    assert_eq!(
        nrp_41_reports.len(),
        28,
        "Expected 28 nrp_41 report entries"
    );
    assert_eq!(
        nrp_51_reports.len(),
        12,
        "Expected 12 nrp_51 report entries"
    );
    assert_eq!(
        nrsf_03_reports.len(),
        8,
        "Expected 8 nrsf_03 report entries"
    );

    // Each report name appears twice (once for each file type: xml/csv or csv/txt)
    let expected_nrp_41_names = [
        "garantia_hipotecaria",
        "garantia_fiduciaria",
        "persona",
        "garantia_aval",
        "garantia_prenda",
        "garantia_bono",
        "garantia_poliza",
        "garantia_fondo",
        "referencia_gasto",
        "referencia_unidad",
        "referencia_cancelada",
        "socios_sociedades",
        "garantia_prendaria",
        "junta_directiva",
    ];

    for expected_name in &expected_nrp_41_names {
        let count = nrp_41_reports
            .iter()
            .filter(|r| r.name == *expected_name)
            .count();
        assert_eq!(
            count, 2,
            "Expected 2 entries for {} in nrp_41, found {}",
            expected_name, count
        );
    }

    let expected_nrp_51_names = [
        "deposito_extranjero",
        "dato_extracontable",
        "titulo_valor_extranjero",
        "aval_garantizado",
        "prestamo_garantizado",
        "deuda_subordinada",
    ];

    for expected_name in &expected_nrp_51_names {
        let count = nrp_51_reports
            .iter()
            .filter(|r| r.name == *expected_name)
            .count();
        assert_eq!(
            count, 2,
            "Expected 2 entries for {} in nrp_51, found {}",
            expected_name, count
        );
    }

    let expected_nrsf_03_names = ["documentos_clientes", "titulares", "agencias", "productos"];

    for expected_name in &expected_nrsf_03_names {
        let count = nrsf_03_reports
            .iter()
            .filter(|r| r.name == *expected_name)
            .count();
        assert_eq!(
            count, 2,
            "Expected 2 entries for {} in nrsf_03, found {}",
            expected_name, count
        );
    }

    let first_report = reports
        .iter()
        .find(|r| r.name == "garantia_hipotecaria" && r.files.iter().any(|f| f.extension == "xml"))
        .unwrap();
    assert_eq!(first_report.files.len(), 1);
    assert_eq!(first_report.files[0].extension, "xml");
    assert!(
        first_report.files[0]
            .path_in_bucket
            .contains("garantia_hipotecaria.xml")
    );
}

#[test]
fn test_parse_single_report() {
    let json_data = r#"{
        "data": {
            "logsForRun": {
                "events": [
                    {
                        "__typename": "MaterializationEvent",
                        "metadataEntries": [
                            {
                                "__typename": "JsonMetadataEntry",
                                "label": "report",
                                "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/test-id/nrp_41/test_report.xml\", \"type\": \"xml\"}], \"name\": \"test_report\", \"norm\": \"nrp_41\"}"
                            }
                        ]
                    }
                ]
            }
        }
    }"#;

    let response: serde_json::Value = serde_json::from_str(json_data).unwrap();
    let reports = parse_reports_from_logs_response(&response);

    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].name, "test_report");
    assert_eq!(reports[0].norm, "nrp_41");
    assert_eq!(reports[0].files.len(), 1);
    assert_eq!(reports[0].files[0].extension, "xml");
    assert_eq!(
        reports[0].files[0].path_in_bucket,
        "reports/test-id/nrp_41/test_report.xml"
    );
}

#[test]
fn test_ignore_non_report_metadata_entries() {
    let json_data = r#"{
        "data": {
            "logsForRun": {
                "events": [
                    {
                        "__typename": "MaterializationEvent",
                        "metadataEntries": [
                            {
                                "__typename": "JsonMetadataEntry",
                                "label": "other_label",
                                "jsonString": "{\"files\": [{\"path_in_bucket\": \"should/not/parse\", \"type\": \"xml\"}], \"name\": \"ignored\", \"norm\": \"nrp_41\"}"
                            },
                            {
                                "__typename": "TextMetadataEntry",
                                "label": "report",
                                "text": "not json"
                            },
                            {
                                "__typename": "JsonMetadataEntry",
                                "label": "report",
                                "jsonString": "{\"files\": [{\"path_in_bucket\": \"should/parse\", \"type\": \"csv\"}], \"name\": \"valid_report\", \"norm\": \"nrp_51\"}"
                            }
                        ]
                    }
                ]
            }
        }
    }"#;

    let response: serde_json::Value = serde_json::from_str(json_data).unwrap();
    let reports = parse_reports_from_logs_response(&response);

    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].name, "valid_report");
}

#[test]
fn test_ignore_non_materialization_events() {
    let json_data = r#"{
        "data": {
            "logsForRun": {
                "events": [
                    {
                        "__typename": "LogMessageEvent"
                    },
                    {
                        "__typename": "ExecutionStepStartEvent"
                    },
                    {
                        "__typename": "MaterializationEvent",
                        "metadataEntries": [
                            {
                                "__typename": "JsonMetadataEntry",
                                "label": "report",
                                "jsonString": "{\"files\": [{\"path_in_bucket\": \"path\", \"type\": \"xml\"}], \"name\": \"valid\", \"norm\": \"nrp_41\"}"
                            }
                        ]
                    },
                    {
                        "__typename": "ExecutionStepSuccessEvent"
                    }
                ]
            }
        }
    }"#;

    let response: serde_json::Value = serde_json::from_str(json_data).unwrap();
    let reports = parse_reports_from_logs_response(&response);

    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].name, "valid");
}

fn get_test_logs_for_run_response() -> &'static str {
    r#"{
  "data": {
    "logsForRun": {
      "events": [
        {"__typename": "AssetMaterializationPlannedEvent"},
        {"__typename": "RunEnqueuedEvent"},
        {"__typename": "RunStartingEvent"},
        {"__typename": "EngineEvent"},
        {"__typename": "RunStartEvent"},
        {"__typename": "StepWorkerStartingEvent"},
        {"__typename": "StepWorkerStartedEvent"},
        {"__typename": "ResourceInitStartedEvent"},
        {"__typename": "ResourceInitSuccessEvent"},
        {"__typename": "LogsCapturedEvent"},
        {"__typename": "ExecutionStepStartEvent"},
        {"__typename": "LogMessageEvent"},
        {"__typename": "ExecutionStepOutputEvent"},
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_41/garantia_hipotecaria.xml\", \"type\": \"xml\"}], \"name\": \"garantia_hipotecaria\", \"norm\": \"nrp_41\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_41/garantia_fiduciaria.xml\", \"type\": \"xml\"}], \"name\": \"garantia_fiduciaria\", \"norm\": \"nrp_41\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_41/garantia_fiduciaria.csv\", \"type\": \"csv\"}], \"name\": \"garantia_fiduciaria\", \"norm\": \"nrp_41\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_41/persona.csv\", \"type\": \"csv\"}], \"name\": \"persona\", \"norm\": \"nrp_41\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_41/garantia_hipotecaria.csv\", \"type\": \"csv\"}], \"name\": \"garantia_hipotecaria\", \"norm\": \"nrp_41\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_41/persona.xml\", \"type\": \"xml\"}], \"name\": \"persona\", \"norm\": \"nrp_41\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_41/garantia_aval.csv\", \"type\": \"csv\"}], \"name\": \"garantia_aval\", \"norm\": \"nrp_41\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_41/garantia_aval.xml\", \"type\": \"xml\"}], \"name\": \"garantia_aval\", \"norm\": \"nrp_41\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_41/garantia_prenda.xml\", \"type\": \"xml\"}], \"name\": \"garantia_prenda\", \"norm\": \"nrp_41\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_41/garantia_bono.xml\", \"type\": \"xml\"}], \"name\": \"garantia_bono\", \"norm\": \"nrp_41\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_41/garantia_poliza.csv\", \"type\": \"csv\"}], \"name\": \"garantia_poliza\", \"norm\": \"nrp_41\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_41/garantia_fondo.csv\", \"type\": \"csv\"}], \"name\": \"garantia_fondo\", \"norm\": \"nrp_41\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_41/garantia_prenda.csv\", \"type\": \"csv\"}], \"name\": \"garantia_prenda\", \"norm\": \"nrp_41\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_41/garantia_bono.csv\", \"type\": \"csv\"}], \"name\": \"garantia_bono\", \"norm\": \"nrp_41\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_41/garantia_fondo.xml\", \"type\": \"xml\"}], \"name\": \"garantia_fondo\", \"norm\": \"nrp_41\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_41/garantia_poliza.xml\", \"type\": \"xml\"}], \"name\": \"garantia_poliza\", \"norm\": \"nrp_41\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_41/referencia_gasto.csv\", \"type\": \"csv\"}], \"name\": \"referencia_gasto\", \"norm\": \"nrp_41\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_41/referencia_gasto.xml\", \"type\": \"xml\"}], \"name\": \"referencia_gasto\", \"norm\": \"nrp_41\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_41/referencia_unidad.xml\", \"type\": \"xml\"}], \"name\": \"referencia_unidad\", \"norm\": \"nrp_41\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_41/referencia_unidad.csv\", \"type\": \"csv\"}], \"name\": \"referencia_unidad\", \"norm\": \"nrp_41\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_41/referencia_cancelada.csv\", \"type\": \"csv\"}], \"name\": \"referencia_cancelada\", \"norm\": \"nrp_41\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_41/referencia_cancelada.xml\", \"type\": \"xml\"}], \"name\": \"referencia_cancelada\", \"norm\": \"nrp_41\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_41/socios_sociedades.csv\", \"type\": \"csv\"}], \"name\": \"socios_sociedades\", \"norm\": \"nrp_41\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_41/garantia_prendaria.csv\", \"type\": \"csv\"}], \"name\": \"garantia_prendaria\", \"norm\": \"nrp_41\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_41/socios_sociedades.xml\", \"type\": \"xml\"}], \"name\": \"socios_sociedades\", \"norm\": \"nrp_41\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_41/garantia_prendaria.xml\", \"type\": \"xml\"}], \"name\": \"garantia_prendaria\", \"norm\": \"nrp_41\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_41/junta_directiva.xml\", \"type\": \"xml\"}], \"name\": \"junta_directiva\", \"norm\": \"nrp_41\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_41/junta_directiva.csv\", \"type\": \"csv\"}], \"name\": \"junta_directiva\", \"norm\": \"nrp_41\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_51/deposito_extranjero.csv\", \"type\": \"csv\"}], \"name\": \"deposito_extranjero\", \"norm\": \"nrp_51\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_51/dato_extracontable.csv\", \"type\": \"csv\"}], \"name\": \"dato_extracontable\", \"norm\": \"nrp_51\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_51/deposito_extranjero.xml\", \"type\": \"xml\"}], \"name\": \"deposito_extranjero\", \"norm\": \"nrp_51\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_51/titulo_valor_extranjero.csv\", \"type\": \"csv\"}], \"name\": \"titulo_valor_extranjero\", \"norm\": \"nrp_51\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_51/dato_extracontable.xml\", \"type\": \"xml\"}], \"name\": \"dato_extracontable\", \"norm\": \"nrp_51\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_51/titulo_valor_extranjero.xml\", \"type\": \"xml\"}], \"name\": \"titulo_valor_extranjero\", \"norm\": \"nrp_51\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_51/aval_garantizado.xml\", \"type\": \"xml\"}], \"name\": \"aval_garantizado\", \"norm\": \"nrp_51\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_51/prestamo_garantizado.csv\", \"type\": \"csv\"}], \"name\": \"prestamo_garantizado\", \"norm\": \"nrp_51\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_51/deuda_subordinada.csv\", \"type\": \"csv\"}], \"name\": \"deuda_subordinada\", \"norm\": \"nrp_51\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_51/aval_garantizado.csv\", \"type\": \"csv\"}], \"name\": \"aval_garantizado\", \"norm\": \"nrp_51\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_51/prestamo_garantizado.xml\", \"type\": \"xml\"}], \"name\": \"prestamo_garantizado\", \"norm\": \"nrp_51\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrp_51/deuda_subordinada.xml\", \"type\": \"xml\"}], \"name\": \"deuda_subordinada\", \"norm\": \"nrp_51\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrsf_03/documentos_clientes.csv\", \"type\": \"csv\"}], \"name\": \"documentos_clientes\", \"norm\": \"nrsf_03\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrsf_03/documentos_clientes.txt\", \"type\": \"txt\"}], \"name\": \"documentos_clientes\", \"norm\": \"nrsf_03\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrsf_03/titulares.csv\", \"type\": \"csv\"}], \"name\": \"titulares\", \"norm\": \"nrsf_03\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrsf_03/titulares.txt\", \"type\": \"txt\"}], \"name\": \"titulares\", \"norm\": \"nrsf_03\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrsf_03/agencias.csv\", \"type\": \"csv\"}], \"name\": \"agencias\", \"norm\": \"nrsf_03\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrsf_03/agencias.txt\", \"type\": \"txt\"}], \"name\": \"agencias\", \"norm\": \"nrsf_03\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrsf_03/productos.csv\", \"type\": \"csv\"}], \"name\": \"productos\", \"norm\": \"nrsf_03\"}"
            }
          ]
        },
        {
          "__typename": "MaterializationEvent",
          "metadataEntries": [
            {
              "__typename": "JsonMetadataEntry",
              "label": "report",
              "jsonString": "{\"files\": [{\"path_in_bucket\": \"reports/33a22012-2e51-4b29-96cf-bb393852340e/nrsf_03/productos.txt\", \"type\": \"txt\"}], \"name\": \"productos\", \"norm\": \"nrsf_03\"}"
            }
          ]
        },
        {"__typename": "ExecutionStepSuccessEvent"},
        {"__typename": "RunSuccessEvent"}
      ]
    }
  }
}"#
}
