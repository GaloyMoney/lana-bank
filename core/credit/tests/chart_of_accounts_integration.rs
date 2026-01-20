mod helpers;

use es_entity::clock::{ArtificialClockConfig, ClockHandle};
use rand::Rng;

use authz::dummy::DummySubject;
use cala_ledger::{CalaLedger, CalaLedgerConfig};
use cloud_storage::{Storage, config::StorageConfig};
use core_accounting::CoreAccounting;
use core_credit::*;
use document_storage::DocumentStorage;
use helpers::{BASE_ACCOUNTS_CSV, action, default_accounting_base_config, event, object};
use public_id::PublicIds;

const CREDIT_ACCOUNTS_CSV: &str = r#"
81,,,Facility Omnibus Parent,,
82,,,Collateral Omnibus Parent,,
83,,,Facility Parent,,
84,,,Collateral Parent,,
11,,,Disbursed Receivable Parent,,
12,,,Interest Receivable Parent,,
41,,,Interest Income Parent,,
42,,,Fee Income Parent,,
13,,,Payment Holding Parent,,
85,,,Liquidation Proceeds Omnibus Parent,,
86,,,Collateral In Liquidation Parent,,
87,,,Uncovered Outstanding Parent,,
88,,,Payments Made Omnibus Parent,,
89,,,Interest Added To Obligations Omnibus Parent,,
"#;

#[tokio::test]
async fn chart_of_accounts_integration() -> anyhow::Result<()> {
    let pool = helpers::init_pool().await?;
    let (clock, _ctrl) = ClockHandle::artificial(ArtificialClockConfig::manual());
    let outbox =
        obix::Outbox::<event::DummyEvent>::init(&pool, obix::MailboxConfig::builder().build()?)
            .await?;
    let authz = authz::dummy::DummyPerms::<action::DummyAction, object::DummyObject>::new();
    let storage = Storage::new(&StorageConfig::default());
    let document_storage = DocumentStorage::new(&pool, &storage, clock.clone());

    let governance = governance::Governance::new(&pool, &authz, &outbox, clock.clone());
    let public_ids = public_id::PublicIds::new(&pool);
    let customers = core_customer::Customers::new(
        &pool,
        &authz,
        &outbox,
        document_storage,
        public_ids,
        clock.clone(),
    );
    let custody = core_custody::CoreCustody::init(
        &pool,
        &authz,
        helpers::custody_config(),
        &outbox,
        clock.clone(),
    )
    .await?;

    let cala_config = CalaLedgerConfig::builder()
        .pool(pool.clone())
        .exec_migrations(false)
        .build()?;
    let cala = CalaLedger::init(cala_config).await?;

    let mut jobs = job::Jobs::init(
        job::JobSvcConfig::builder()
            .pool(pool.clone())
            .build()
            .unwrap(),
    )
    .await?;

    let journal_id = helpers::init_journal(&cala).await?;
    let public_ids = PublicIds::new(&pool);
    let price = core_price::Price::init(&mut jobs, &outbox).await?;

    let credit = CoreCredit::init(
        &pool,
        Default::default(),
        &governance,
        &mut jobs,
        &authz,
        &customers,
        &custody,
        &price,
        &outbox,
        &cala,
        journal_id,
        &public_ids,
    )
    .await?;

    let accounting_document_storage = DocumentStorage::new(&pool, &storage, clock.clone());
    let accounting = CoreAccounting::new(
        &pool,
        &authz,
        &cala,
        journal_id,
        accounting_document_storage,
        &mut jobs,
        &outbox,
    );
    let chart_ref = format!("ref-{:08}", rand::rng().random_range(0..10000));
    let chart_id = accounting
        .chart_of_accounts()
        .create_chart(&DummySubject, "Test chart".to_string(), chart_ref.clone())
        .await?
        .id;

    let import = format!("{}{}", BASE_ACCOUNTS_CSV, CREDIT_ACCOUNTS_CSV);
    let base_config = default_accounting_base_config();
    let (chart, _) = accounting
        .chart_of_accounts()
        .import_from_csv_with_base_config(&DummySubject, &chart_ref, import, base_config.clone())
        .await?;

    let off_balance_sheet_code = "8".parse::<core_accounting::AccountCode>().unwrap();
    let off_balance_sheet_account_set_id = cala
        .account_sets()
        .find(
            chart
                .account_set_id_from_code(&off_balance_sheet_code)
                .unwrap(),
        )
        .await?
        .id;

    let assets_code = "1".parse::<core_accounting::AccountCode>().unwrap();
    let assets_account_set_id = cala
        .account_sets()
        .find(chart.account_set_id_from_code(&assets_code).unwrap())
        .await?
        .id;

    credit.chart_of_accounts_integrations()
        .set_config(
            &DummySubject,
            &chart,
            ChartOfAccountsIntegrationConfig {
                chart_of_accounts_id: chart_id,
                chart_of_account_facility_omnibus_parent_code: "81".parse().unwrap(),
                chart_of_account_collateral_omnibus_parent_code: "82".parse().unwrap(),
                chart_of_account_liquidation_proceeds_omnibus_parent_code: "85".parse().unwrap(),
                chart_of_account_payments_made_omnibus_parent_code: "88".parse().unwrap(),
                chart_of_account_interest_added_to_obligations_omnibus_parent_code: "89".parse().unwrap(),
                chart_of_account_facility_parent_code: "83".parse().unwrap(),
                chart_of_account_collateral_parent_code: "84".parse().unwrap(),
                chart_of_account_collateral_in_liquidation_parent_code: "86".parse().unwrap(),
                chart_of_account_interest_income_parent_code: "41".parse().unwrap(),
                chart_of_account_fee_income_parent_code: "42".parse().unwrap(),
                chart_of_account_payment_holding_parent_code: "13".parse().unwrap(),
                chart_of_account_uncovered_outstanding_parent_code: "87".parse().unwrap(),
                chart_of_account_short_term_individual_disbursed_receivable_parent_code: "11".parse().unwrap(),
                chart_of_account_short_term_government_entity_disbursed_receivable_parent_code:
                    "11".parse().unwrap(),
                chart_of_account_short_term_private_company_disbursed_receivable_parent_code:
                    "11".parse().unwrap(),
                chart_of_account_short_term_bank_disbursed_receivable_parent_code: "11".parse().unwrap(),
                chart_of_account_short_term_financial_institution_disbursed_receivable_parent_code:
                    "11".parse().unwrap(),
                chart_of_account_short_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code:
                    "11".parse().unwrap(),
                chart_of_account_short_term_non_domiciled_company_disbursed_receivable_parent_code:
                    "11".parse().unwrap(),
                chart_of_account_long_term_individual_disbursed_receivable_parent_code: "11".parse().unwrap(),
                chart_of_account_long_term_government_entity_disbursed_receivable_parent_code:
                    "11".parse().unwrap(),
                chart_of_account_long_term_private_company_disbursed_receivable_parent_code:
                    "11".parse().unwrap(),
                chart_of_account_long_term_bank_disbursed_receivable_parent_code: "11".parse().unwrap(),
                chart_of_account_long_term_financial_institution_disbursed_receivable_parent_code:
                    "11".parse().unwrap(),
                chart_of_account_long_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code:
                    "11".parse().unwrap(),
                chart_of_account_long_term_non_domiciled_company_disbursed_receivable_parent_code:
                    "11".parse().unwrap(),
                chart_of_account_short_term_individual_interest_receivable_parent_code: "12".parse().unwrap(),
                chart_of_account_short_term_government_entity_interest_receivable_parent_code:
                    "12".parse().unwrap(),
                chart_of_account_short_term_private_company_interest_receivable_parent_code:
                    "12".parse().unwrap(),
                chart_of_account_short_term_bank_interest_receivable_parent_code: "12".parse().unwrap(),
                chart_of_account_short_term_financial_institution_interest_receivable_parent_code:
                    "12".parse().unwrap(),
                chart_of_account_short_term_foreign_agency_or_subsidiary_interest_receivable_parent_code:
                    "12".parse().unwrap(),
                chart_of_account_short_term_non_domiciled_company_interest_receivable_parent_code:
                    "12".parse().unwrap(),
                chart_of_account_long_term_individual_interest_receivable_parent_code: "12".parse().unwrap(),
                chart_of_account_long_term_government_entity_interest_receivable_parent_code:
                    "12".parse().unwrap(),
                chart_of_account_long_term_private_company_interest_receivable_parent_code:
                    "12".parse().unwrap(),
                chart_of_account_long_term_bank_interest_receivable_parent_code: "12".parse().unwrap(),
                chart_of_account_long_term_financial_institution_interest_receivable_parent_code:
                    "12".parse().unwrap(),
                chart_of_account_long_term_foreign_agency_or_subsidiary_interest_receivable_parent_code:
                    "12".parse().unwrap(),
                chart_of_account_long_term_non_domiciled_company_interest_receivable_parent_code:
                    "12".parse().unwrap(),
                chart_of_account_overdue_individual_disbursed_receivable_parent_code: "11".parse().unwrap(),
                chart_of_account_overdue_government_entity_disbursed_receivable_parent_code:
                    "11".parse().unwrap(),
                chart_of_account_overdue_private_company_disbursed_receivable_parent_code:
                    "11".parse().unwrap(),
                chart_of_account_overdue_bank_disbursed_receivable_parent_code: "11".parse().unwrap(),
                chart_of_account_overdue_financial_institution_disbursed_receivable_parent_code:
                    "11".parse().unwrap(),
                chart_of_account_overdue_foreign_agency_or_subsidiary_disbursed_receivable_parent_code:
                    "11".parse().unwrap(),
                chart_of_account_overdue_non_domiciled_company_disbursed_receivable_parent_code:
                    "11".parse().unwrap(),
            },
        )
        .await?;

    let off_balance_sheet_account_sets = cala
        .account_sets()
        .list_members_by_created_at(off_balance_sheet_account_set_id, Default::default())
        .await?;

    assert_eq!(off_balance_sheet_account_sets.entities.len(), 9);

    let assets_account_sets = cala
        .account_sets()
        .list_members_by_created_at(assets_account_set_id, Default::default())
        .await?;

    assert_eq!(assets_account_sets.entities.len(), 3);

    let chart_ref = format!("other-ref-{:08}", rand::rng().random_range(0..10000));
    let chart_id = accounting
        .chart_of_accounts()
        .create_chart(
            &DummySubject,
            "Other Test chart".to_string(),
            chart_ref.to_string(),
        )
        .await?
        .id;

    // let import = r#"
    //     81,,,Other Facility Omnibus Parent,,
    //     82,,,Other Collateral Omnibus Parent,,
    //     83,,,Other Facility Parent,,
    //     84,,,Other Collateral Parent,,
    //     11,,,Other Disbursed Receivable Parent,,
    //     12,,,Other Interest Receivable Parent,,
    //     41,,,Other Interest Income Parent,,
    //     42,,,Other Fee Income Parent,,
    //     13,,,Other Payment Holding Parent,,
    //     85,,,Other Liquidation Proceeds Omnibus Parent,,
    //     86,,,Other Collateral In Liquidation Parent,,
    //     87,,,Other Uncovered Outstanding Parent,,
    //     88,,,Other Payments Made Omnibus Parent,,
    //     89,,,Other Interest Added To Obligations Omnibus Parent,,
    //     "#
    // .to_string();
    let import = format!(
        "{}{}",
        BASE_ACCOUNTS_CSV,
        r#"
    81,,,Other Facility Omnibus Parent,,
    82,,,Other Collateral Omnibus Parent,,
    83,,,Other Facility Parent,,
    84,,,Other Collateral Parent,,
    11,,,Other Disbursed Receivable Parent,,
    12,,,Other Interest Receivable Parent,,
    41,,,Other Interest Income Parent,,
    42,,,Other Fee Income Parent,,
    13,,,Other Payment Holding Parent,,
    85,,,Other Liquidation Proceeds Omnibus Parent,,
    86,,,Other Collateral In Liquidation Parent,,
    87,,,Other Uncovered Outstanding Parent,,
    88,,,Other Payments Made Omnibus Parent,,
    89,,,Other Interest Added To Obligations Omnibus Parent,,
    "#
    );
    let (chart, _) = accounting
        .chart_of_accounts()
        .import_from_csv_with_base_config(&DummySubject, &chart_ref, import, base_config)
        .await?;

    let res = credit.chart_of_accounts_integrations()
        .set_config(
            &DummySubject,
            &chart,
            ChartOfAccountsIntegrationConfig {
                chart_of_accounts_id: chart_id,
                chart_of_account_facility_omnibus_parent_code: "81".parse().unwrap(),
                chart_of_account_collateral_omnibus_parent_code: "82".parse().unwrap(),
                chart_of_account_liquidation_proceeds_omnibus_parent_code: "85".parse().unwrap(),
                chart_of_account_payments_made_omnibus_parent_code: "88".parse().unwrap(),
                chart_of_account_interest_added_to_obligations_omnibus_parent_code: "89".parse().unwrap(),
                chart_of_account_facility_parent_code: "83".parse().unwrap(),
                chart_of_account_collateral_parent_code: "84".parse().unwrap(),
                chart_of_account_collateral_in_liquidation_parent_code: "86".parse().unwrap(),
                chart_of_account_interest_income_parent_code: "41".parse().unwrap(),
                chart_of_account_fee_income_parent_code: "42".parse().unwrap(),
                chart_of_account_payment_holding_parent_code: "13".parse().unwrap(),
                chart_of_account_uncovered_outstanding_parent_code: "87".parse().unwrap(),
                chart_of_account_short_term_individual_disbursed_receivable_parent_code: "11".parse().unwrap(),
                chart_of_account_short_term_government_entity_disbursed_receivable_parent_code:
                    "11".parse().unwrap(),
                chart_of_account_short_term_private_company_disbursed_receivable_parent_code:
                    "11".parse().unwrap(),
                chart_of_account_short_term_bank_disbursed_receivable_parent_code: "11".parse().unwrap(),
                chart_of_account_short_term_financial_institution_disbursed_receivable_parent_code:
                    "11".parse().unwrap(),
                chart_of_account_short_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code:
                    "11".parse().unwrap(),
                chart_of_account_short_term_non_domiciled_company_disbursed_receivable_parent_code:
                    "11".parse().unwrap(),
                chart_of_account_long_term_individual_disbursed_receivable_parent_code: "11".parse().unwrap(),
                chart_of_account_long_term_government_entity_disbursed_receivable_parent_code:
                    "11".parse().unwrap(),
                chart_of_account_long_term_private_company_disbursed_receivable_parent_code:
                    "11".parse().unwrap(),
                chart_of_account_long_term_bank_disbursed_receivable_parent_code: "11".parse().unwrap(),
                chart_of_account_long_term_financial_institution_disbursed_receivable_parent_code:
                    "11".parse().unwrap(),
                chart_of_account_long_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code:
                    "11".parse().unwrap(),
                chart_of_account_long_term_non_domiciled_company_disbursed_receivable_parent_code:
                    "11".parse().unwrap(),
                chart_of_account_short_term_individual_interest_receivable_parent_code: "12".parse().unwrap(),
                chart_of_account_short_term_government_entity_interest_receivable_parent_code:
                    "12".parse().unwrap(),
                chart_of_account_short_term_private_company_interest_receivable_parent_code:
                    "12".parse().unwrap(),
                chart_of_account_short_term_bank_interest_receivable_parent_code: "12".parse().unwrap(),
                chart_of_account_short_term_financial_institution_interest_receivable_parent_code:
                    "12".parse().unwrap(),
                chart_of_account_short_term_foreign_agency_or_subsidiary_interest_receivable_parent_code:
                    "12".parse().unwrap(),
                chart_of_account_short_term_non_domiciled_company_interest_receivable_parent_code:
                    "12".parse().unwrap(),
                chart_of_account_long_term_individual_interest_receivable_parent_code: "12".parse().unwrap(),
                chart_of_account_long_term_government_entity_interest_receivable_parent_code:
                    "12".parse().unwrap(),
                chart_of_account_long_term_private_company_interest_receivable_parent_code:
                    "12".parse().unwrap(),
                chart_of_account_long_term_bank_interest_receivable_parent_code: "12".parse().unwrap(),
                chart_of_account_long_term_financial_institution_interest_receivable_parent_code:
                    "12".parse().unwrap(),
                chart_of_account_long_term_foreign_agency_or_subsidiary_interest_receivable_parent_code:
                    "12".parse().unwrap(),
                chart_of_account_long_term_non_domiciled_company_interest_receivable_parent_code:
                    "12".parse().unwrap(),
                chart_of_account_overdue_individual_disbursed_receivable_parent_code: "11".parse().unwrap(),
                chart_of_account_overdue_government_entity_disbursed_receivable_parent_code:
                    "11".parse().unwrap(),
                chart_of_account_overdue_private_company_disbursed_receivable_parent_code:
                    "11".parse().unwrap(),
                chart_of_account_overdue_bank_disbursed_receivable_parent_code: "11".parse().unwrap(),
                chart_of_account_overdue_financial_institution_disbursed_receivable_parent_code:
                    "11".parse().unwrap(),
                chart_of_account_overdue_foreign_agency_or_subsidiary_disbursed_receivable_parent_code:
                    "11".parse().unwrap(),
                chart_of_account_overdue_non_domiciled_company_disbursed_receivable_parent_code:
                    "11".parse().unwrap(),
            },
        )
        .await;

    assert!(matches!(
        res,
        Err(core_credit::ChartOfAccountsIntegrationError::CreditConfigAlreadyExists)
    ));

    Ok(())
}
