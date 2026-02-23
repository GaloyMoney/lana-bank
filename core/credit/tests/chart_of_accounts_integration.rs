mod helpers;

use authz::dummy::DummySubject;
use cala_ledger::{CalaLedger, CalaLedgerConfig};
use cloud_storage::{Storage, config::StorageConfig};
use core_accounting::{AccountCode, CalaAccountSetId, CoreAccounting};
use core_credit::*;
use document_storage::DocumentStorage;
use es_entity::clock::{ArtificialClockConfig, ClockHandle};
use std::collections::HashMap;

use helpers::{
    BASE_ACCOUNTS_CSV, action, assert_attached_for_code, default_accounting_base_config, event,
    object, resolve_account_set_ids, resolve_omnibus_account_set_ids,
};
use public_id::PublicIds;
use rand::RngExt;

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

async fn assert_summary_pairs(
    cala: &CalaLedger,
    chart: &core_accounting::Chart,
    summary_ids: &HashMap<&'static str, CalaAccountSetId>,
    pairs: &[(&AccountCode, CreditSummaryAccountSetSpec)],
) -> anyhow::Result<()> {
    for (code, spec) in pairs {
        let id = *summary_ids
            .get(spec.external_ref)
            .expect("missing summary account set ref");
        assert_attached_for_code(cala, chart, code, id).await?;
    }
    Ok(())
}

async fn assert_omnibus_pairs(
    cala: &CalaLedger,
    chart: &core_accounting::Chart,
    omnibus_ids: &HashMap<&'static str, CalaAccountSetId>,
    pairs: &[(&AccountCode, CreditOmnibusAccountSetSpec)],
) -> anyhow::Result<()> {
    for (code, spec) in pairs {
        let id = *omnibus_ids
            .get(spec.account_set_ref)
            .expect("missing omnibus account set ref");
        assert_attached_for_code(cala, chart, code, id).await?;
    }
    Ok(())
}

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
    let mut jobs = job::Jobs::init(
        job::JobSvcConfig::builder()
            .pool(pool.clone())
            .build()
            .unwrap(),
    )
    .await?;
    let custody = core_custody::CoreCustody::init(
        &pool,
        &authz,
        helpers::custody_encryption_config(),
        helpers::custody_config(),
        &outbox,
        &mut jobs,
        clock.clone(),
    )
    .await?;

    let cala_config = CalaLedgerConfig::builder()
        .pool(pool.clone())
        .exec_migrations(false)
        .build()?;
    let cala = CalaLedger::init(cala_config).await?;

    let journal_id = helpers::init_journal(&cala).await?;
    let public_ids = PublicIds::new(&pool);
    let price = core_price::Price::init(
        core_price::PriceConfig { providers: vec![] },
        &mut jobs,
        &outbox,
    )
    .await?;
    let exposed_domain_configs =
        helpers::init_read_only_exposed_domain_configs(&pool, &authz).await?;
    // Required to prevent the case there is an attempt to remove an account set member from
    // an account set that no longer exists.
    domain_config::DomainConfigTestUtils::clear_config_by_key(
        &pool,
        "credit-chart-of-accounts-integration",
    )
    .await?;
    let internal_domain_configs = helpers::init_internal_domain_configs(&pool).await?;
    let credit = CoreCredit::init(
        &pool,
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
        &exposed_domain_configs,
        &internal_domain_configs,
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
    let chart_ref = format!("ref-{:010}", rand::rng().random_range(0..10_000_000_000u64));
    let chart_id = accounting
        .chart_of_accounts()
        .create_chart(&DummySubject, "Test chart".to_string(), chart_ref.clone())
        .await?
        .id;

    let (balance_sheet_name, pl_name, tb_name) =
        helpers::create_test_statements(&accounting).await?;

    let import = format!("{}{}", BASE_ACCOUNTS_CSV, CREDIT_ACCOUNTS_CSV);
    let base_config = default_accounting_base_config();
    let chart = accounting
        .import_csv_with_base_config(
            &DummySubject,
            &chart_ref,
            import,
            base_config.clone(),
            &balance_sheet_name,
            &pl_name,
            &tb_name,
        )
        .await?;

    let off_balance_sheet_code = "8".parse::<core_accounting::AccountCode>().unwrap();
    let off_balance_sheet_account_set_id = cala
        .account_sets()
        .find(
            chart
                .maybe_account_set_id_from_code(&off_balance_sheet_code)
                .unwrap(),
        )
        .await?
        .id;

    let assets_code = "1".parse::<core_accounting::AccountCode>().unwrap();
    let assets_account_set_id = cala
        .account_sets()
        .find(chart.maybe_account_set_id_from_code(&assets_code).unwrap())
        .await?
        .id;

    let chart_of_accounts_config = ChartOfAccountsIntegrationConfig {
        chart_of_accounts_id: chart_id,
        chart_of_account_facility_omnibus_parent_code: "81".parse().unwrap(),
        chart_of_account_collateral_omnibus_parent_code: "82".parse().unwrap(),
        chart_of_account_liquidation_proceeds_omnibus_parent_code: "85".parse().unwrap(),
        chart_of_account_payments_made_omnibus_parent_code: "88".parse().unwrap(),
        chart_of_account_interest_added_to_obligations_omnibus_parent_code: "89".parse().unwrap(),
        chart_of_account_facility_parent_code: "83".parse().unwrap(),
        chart_of_account_collateral_parent_code: "84".parse().unwrap(),
        chart_of_account_collateral_in_liquidation_parent_code: "86".parse().unwrap(),
        chart_of_account_liquidated_collateral_parent_code: "86".parse().unwrap(),
        chart_of_account_proceeds_from_liquidation_parent_code: "85".parse().unwrap(),
        chart_of_account_interest_income_parent_code: "41".parse().unwrap(),
        chart_of_account_fee_income_parent_code: "42".parse().unwrap(),
        chart_of_account_payment_holding_parent_code: "13".parse().unwrap(),
        chart_of_account_uncovered_outstanding_parent_code: "87".parse().unwrap(),
        chart_of_account_disbursed_defaulted_parent_code: "11".parse().unwrap(),
        chart_of_account_interest_defaulted_parent_code: "11".parse().unwrap(),
        chart_of_account_short_term_individual_disbursed_receivable_parent_code: "11"
            .parse()
            .unwrap(),
        chart_of_account_short_term_government_entity_disbursed_receivable_parent_code: "11"
            .parse()
            .unwrap(),
        chart_of_account_short_term_private_company_disbursed_receivable_parent_code: "11"
            .parse()
            .unwrap(),
        chart_of_account_short_term_bank_disbursed_receivable_parent_code: "11".parse().unwrap(),
        chart_of_account_short_term_financial_institution_disbursed_receivable_parent_code: "11"
            .parse()
            .unwrap(),
        chart_of_account_short_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code:
            "11".parse().unwrap(),
        chart_of_account_short_term_non_domiciled_company_disbursed_receivable_parent_code: "11"
            .parse()
            .unwrap(),
        chart_of_account_long_term_individual_disbursed_receivable_parent_code: "11"
            .parse()
            .unwrap(),
        chart_of_account_long_term_government_entity_disbursed_receivable_parent_code: "11"
            .parse()
            .unwrap(),
        chart_of_account_long_term_private_company_disbursed_receivable_parent_code: "11"
            .parse()
            .unwrap(),
        chart_of_account_long_term_bank_disbursed_receivable_parent_code: "11".parse().unwrap(),
        chart_of_account_long_term_financial_institution_disbursed_receivable_parent_code: "11"
            .parse()
            .unwrap(),
        chart_of_account_long_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code:
            "11".parse().unwrap(),
        chart_of_account_long_term_non_domiciled_company_disbursed_receivable_parent_code: "11"
            .parse()
            .unwrap(),
        chart_of_account_short_term_individual_interest_receivable_parent_code: "12"
            .parse()
            .unwrap(),
        chart_of_account_short_term_government_entity_interest_receivable_parent_code: "12"
            .parse()
            .unwrap(),
        chart_of_account_short_term_private_company_interest_receivable_parent_code: "12"
            .parse()
            .unwrap(),
        chart_of_account_short_term_bank_interest_receivable_parent_code: "12".parse().unwrap(),
        chart_of_account_short_term_financial_institution_interest_receivable_parent_code: "12"
            .parse()
            .unwrap(),
        chart_of_account_short_term_foreign_agency_or_subsidiary_interest_receivable_parent_code:
            "12".parse().unwrap(),
        chart_of_account_short_term_non_domiciled_company_interest_receivable_parent_code: "12"
            .parse()
            .unwrap(),
        chart_of_account_long_term_individual_interest_receivable_parent_code: "12"
            .parse()
            .unwrap(),
        chart_of_account_long_term_government_entity_interest_receivable_parent_code: "12"
            .parse()
            .unwrap(),
        chart_of_account_long_term_private_company_interest_receivable_parent_code: "12"
            .parse()
            .unwrap(),
        chart_of_account_long_term_bank_interest_receivable_parent_code: "12".parse().unwrap(),
        chart_of_account_long_term_financial_institution_interest_receivable_parent_code: "12"
            .parse()
            .unwrap(),
        chart_of_account_long_term_foreign_agency_or_subsidiary_interest_receivable_parent_code:
            "12".parse().unwrap(),
        chart_of_account_long_term_non_domiciled_company_interest_receivable_parent_code: "12"
            .parse()
            .unwrap(),
        chart_of_account_overdue_individual_disbursed_receivable_parent_code: "11".parse().unwrap(),
        chart_of_account_overdue_government_entity_disbursed_receivable_parent_code: "11"
            .parse()
            .unwrap(),
        chart_of_account_overdue_private_company_disbursed_receivable_parent_code: "11"
            .parse()
            .unwrap(),
        chart_of_account_overdue_bank_disbursed_receivable_parent_code: "11".parse().unwrap(),
        chart_of_account_overdue_financial_institution_disbursed_receivable_parent_code: "11"
            .parse()
            .unwrap(),
        chart_of_account_overdue_foreign_agency_or_subsidiary_disbursed_receivable_parent_code:
            "11".parse().unwrap(),
        chart_of_account_overdue_non_domiciled_company_disbursed_receivable_parent_code: "11"
            .parse()
            .unwrap(),
    };

    credit
        .chart_of_accounts_integrations()
        .set_config(&DummySubject, &chart, chart_of_accounts_config.clone())
        .await?;

    let catalog = CREDIT_ACCOUNT_SET_CATALOG;
    let summary_account_set_ids =
        resolve_account_set_ids(&cala, journal_id, catalog.summary_specs()).await?;
    let omnibus_account_set_ids =
        resolve_omnibus_account_set_ids(&cala, journal_id, catalog.omnibus_specs()).await?;
    let summary = catalog.summary();
    let omnibus = catalog.omnibus();

    let omnibus_pairs = [
        (
            &chart_of_accounts_config.chart_of_account_facility_omnibus_parent_code,
            omnibus.credit_facility_omnibus,
        ),
        (
            &chart_of_accounts_config.chart_of_account_collateral_omnibus_parent_code,
            omnibus.credit_collateral_omnibus,
        ),
        (
            &chart_of_accounts_config.chart_of_account_liquidation_proceeds_omnibus_parent_code,
            omnibus.credit_facility_liquidation_proceeds_omnibus,
        ),
        (
            &chart_of_accounts_config.chart_of_account_payments_made_omnibus_parent_code,
            omnibus.credit_payments_made_omnibus,
        ),
        (
            &chart_of_accounts_config
                .chart_of_account_interest_added_to_obligations_omnibus_parent_code,
            omnibus.credit_interest_added_to_obligations_omnibus,
        ),
    ];
    assert_omnibus_pairs(&cala, &chart, &omnibus_account_set_ids, &omnibus_pairs).await?;

    let summary_parent_pairs = [
        (
            &chart_of_accounts_config.chart_of_account_facility_parent_code,
            summary.credit_facility_remaining,
        ),
        (
            &chart_of_accounts_config.chart_of_account_collateral_parent_code,
            summary.credit_collateral,
        ),
        (
            &chart_of_accounts_config.chart_of_account_collateral_in_liquidation_parent_code,
            summary.credit_facility_collateral_in_liquidation,
        ),
        (
            &chart_of_accounts_config.chart_of_account_liquidated_collateral_parent_code,
            summary.credit_facility_liquidated_collateral,
        ),
        (
            &chart_of_accounts_config.chart_of_account_proceeds_from_liquidation_parent_code,
            summary.credit_facility_proceeds_from_liquidation,
        ),
        (
            &chart_of_accounts_config.chart_of_account_interest_income_parent_code,
            summary.credit_interest_income,
        ),
        (
            &chart_of_accounts_config.chart_of_account_fee_income_parent_code,
            summary.credit_fee_income,
        ),
        (
            &chart_of_accounts_config.chart_of_account_payment_holding_parent_code,
            summary.credit_payment_holding,
        ),
        (
            &chart_of_accounts_config.chart_of_account_uncovered_outstanding_parent_code,
            summary.credit_uncovered_outstanding,
        ),
        (
            &chart_of_accounts_config.chart_of_account_disbursed_defaulted_parent_code,
            summary.credit_disbursed_defaulted,
        ),
        (
            &chart_of_accounts_config.chart_of_account_interest_defaulted_parent_code,
            summary.credit_interest_defaulted,
        ),
    ];
    assert_summary_pairs(
        &cala,
        &chart,
        &summary_account_set_ids,
        &summary_parent_pairs,
    )
    .await?;

    let short_term_disbursed_pairs = [
        (
            &chart_of_accounts_config
                .chart_of_account_short_term_individual_disbursed_receivable_parent_code,
            summary.short_term_individual_disbursed_receivable,
        ),
        (
            &chart_of_accounts_config
                .chart_of_account_short_term_government_entity_disbursed_receivable_parent_code,
            summary.short_term_government_entity_disbursed_receivable,
        ),
        (
            &chart_of_accounts_config
                .chart_of_account_short_term_private_company_disbursed_receivable_parent_code,
            summary.short_term_private_company_disbursed_receivable,
        ),
        (
            &chart_of_accounts_config
                .chart_of_account_short_term_bank_disbursed_receivable_parent_code,
            summary.short_term_bank_disbursed_receivable,
        ),
        (
            &chart_of_accounts_config
                .chart_of_account_short_term_financial_institution_disbursed_receivable_parent_code,
            summary.short_term_financial_institution_disbursed_receivable,
        ),
        (
            &chart_of_accounts_config
                .chart_of_account_short_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code,
            summary.short_term_foreign_agency_or_subsidiary_disbursed_receivable,
        ),
        (
            &chart_of_accounts_config
                .chart_of_account_short_term_non_domiciled_company_disbursed_receivable_parent_code,
            summary.short_term_non_domiciled_company_disbursed_receivable,
        ),
    ];
    assert_summary_pairs(
        &cala,
        &chart,
        &summary_account_set_ids,
        &short_term_disbursed_pairs,
    )
    .await?;

    let long_term_disbursed_pairs = [
        (
            &chart_of_accounts_config
                .chart_of_account_long_term_individual_disbursed_receivable_parent_code,
            summary.long_term_individual_disbursed_receivable,
        ),
        (
            &chart_of_accounts_config
                .chart_of_account_long_term_government_entity_disbursed_receivable_parent_code,
            summary.long_term_government_entity_disbursed_receivable,
        ),
        (
            &chart_of_accounts_config
                .chart_of_account_long_term_private_company_disbursed_receivable_parent_code,
            summary.long_term_private_company_disbursed_receivable,
        ),
        (
            &chart_of_accounts_config
                .chart_of_account_long_term_bank_disbursed_receivable_parent_code,
            summary.long_term_bank_disbursed_receivable,
        ),
        (
            &chart_of_accounts_config
                .chart_of_account_long_term_financial_institution_disbursed_receivable_parent_code,
            summary.long_term_financial_institution_disbursed_receivable,
        ),
        (
            &chart_of_accounts_config
                .chart_of_account_long_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code,
            summary.long_term_foreign_agency_or_subsidiary_disbursed_receivable,
        ),
        (
            &chart_of_accounts_config
                .chart_of_account_long_term_non_domiciled_company_disbursed_receivable_parent_code,
            summary.long_term_non_domiciled_company_disbursed_receivable,
        ),
    ];
    assert_summary_pairs(
        &cala,
        &chart,
        &summary_account_set_ids,
        &long_term_disbursed_pairs,
    )
    .await?;

    let overdue_disbursed_pairs = [
        (
            &chart_of_accounts_config
                .chart_of_account_overdue_individual_disbursed_receivable_parent_code,
            summary.overdue_individual_disbursed_receivable,
        ),
        (
            &chart_of_accounts_config
                .chart_of_account_overdue_government_entity_disbursed_receivable_parent_code,
            summary.overdue_government_entity_disbursed_receivable,
        ),
        (
            &chart_of_accounts_config
                .chart_of_account_overdue_private_company_disbursed_receivable_parent_code,
            summary.overdue_private_company_disbursed_receivable,
        ),
        (
            &chart_of_accounts_config
                .chart_of_account_overdue_bank_disbursed_receivable_parent_code,
            summary.overdue_bank_disbursed_receivable,
        ),
        (
            &chart_of_accounts_config
                .chart_of_account_overdue_financial_institution_disbursed_receivable_parent_code,
            summary.overdue_financial_institution_disbursed_receivable,
        ),
        (
            &chart_of_accounts_config
                .chart_of_account_overdue_foreign_agency_or_subsidiary_disbursed_receivable_parent_code,
            summary.overdue_foreign_agency_or_subsidiary_disbursed_receivable,
        ),
        (
            &chart_of_accounts_config
                .chart_of_account_overdue_non_domiciled_company_disbursed_receivable_parent_code,
            summary.overdue_non_domiciled_company_disbursed_receivable,
        ),
    ];
    assert_summary_pairs(
        &cala,
        &chart,
        &summary_account_set_ids,
        &overdue_disbursed_pairs,
    )
    .await?;

    let short_term_interest_pairs = [
        (
            &chart_of_accounts_config
                .chart_of_account_short_term_individual_interest_receivable_parent_code,
            summary.short_term_individual_interest_receivable,
        ),
        (
            &chart_of_accounts_config
                .chart_of_account_short_term_government_entity_interest_receivable_parent_code,
            summary.short_term_government_entity_interest_receivable,
        ),
        (
            &chart_of_accounts_config
                .chart_of_account_short_term_private_company_interest_receivable_parent_code,
            summary.short_term_private_company_interest_receivable,
        ),
        (
            &chart_of_accounts_config
                .chart_of_account_short_term_bank_interest_receivable_parent_code,
            summary.short_term_bank_interest_receivable,
        ),
        (
            &chart_of_accounts_config
                .chart_of_account_short_term_financial_institution_interest_receivable_parent_code,
            summary.short_term_financial_institution_interest_receivable,
        ),
        (
            &chart_of_accounts_config
                .chart_of_account_short_term_foreign_agency_or_subsidiary_interest_receivable_parent_code,
            summary.short_term_foreign_agency_or_subsidiary_interest_receivable,
        ),
        (
            &chart_of_accounts_config
                .chart_of_account_short_term_non_domiciled_company_interest_receivable_parent_code,
            summary.short_term_non_domiciled_company_interest_receivable,
        ),
    ];
    assert_summary_pairs(
        &cala,
        &chart,
        &summary_account_set_ids,
        &short_term_interest_pairs,
    )
    .await?;

    let long_term_interest_pairs = [
        (
            &chart_of_accounts_config
                .chart_of_account_long_term_individual_interest_receivable_parent_code,
            summary.long_term_individual_interest_receivable,
        ),
        (
            &chart_of_accounts_config
                .chart_of_account_long_term_government_entity_interest_receivable_parent_code,
            summary.long_term_government_entity_interest_receivable,
        ),
        (
            &chart_of_accounts_config
                .chart_of_account_long_term_private_company_interest_receivable_parent_code,
            summary.long_term_private_company_interest_receivable,
        ),
        (
            &chart_of_accounts_config
                .chart_of_account_long_term_bank_interest_receivable_parent_code,
            summary.long_term_bank_interest_receivable,
        ),
        (
            &chart_of_accounts_config
                .chart_of_account_long_term_financial_institution_interest_receivable_parent_code,
            summary.long_term_financial_institution_interest_receivable,
        ),
        (
            &chart_of_accounts_config
                .chart_of_account_long_term_foreign_agency_or_subsidiary_interest_receivable_parent_code,
            summary.long_term_foreign_agency_or_subsidiary_interest_receivable,
        ),
        (
            &chart_of_accounts_config
                .chart_of_account_long_term_non_domiciled_company_interest_receivable_parent_code,
            summary.long_term_non_domiciled_company_interest_receivable,
        ),
    ];
    assert_summary_pairs(
        &cala,
        &chart,
        &summary_account_set_ids,
        &long_term_interest_pairs,
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

    let chart_ref = format!(
        "other-ref-{:010}",
        rand::rng().random_range(0..10_000_000_000u64)
    );
    let chart_id = accounting
        .chart_of_accounts()
        .create_chart(
            &DummySubject,
            "Other Test chart".to_string(),
            chart_ref.to_string(),
        )
        .await?
        .id;

    let (balance_sheet_name2, pl_name2, tb_name2) =
        helpers::create_test_statements(&accounting).await?;

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
    let chart = accounting
        .import_csv_with_base_config(
            &DummySubject,
            &chart_ref,
            import,
            base_config,
            &balance_sheet_name2,
            &pl_name2,
            &tb_name2,
        )
        .await?;
    let chart_of_accounts_integration_config = ChartOfAccountsIntegrationConfig {
        chart_of_accounts_id: chart_id,
        chart_of_account_facility_omnibus_parent_code: "81".parse().unwrap(),
        chart_of_account_collateral_omnibus_parent_code: "82".parse().unwrap(),
        chart_of_account_liquidation_proceeds_omnibus_parent_code: "85".parse().unwrap(),
        chart_of_account_payments_made_omnibus_parent_code: "88".parse().unwrap(),
        chart_of_account_interest_added_to_obligations_omnibus_parent_code: "89".parse().unwrap(),
        chart_of_account_facility_parent_code: "83".parse().unwrap(),
        chart_of_account_collateral_parent_code: "84".parse().unwrap(),
        chart_of_account_collateral_in_liquidation_parent_code: "86".parse().unwrap(),
        chart_of_account_liquidated_collateral_parent_code: "86".parse().unwrap(),
        chart_of_account_proceeds_from_liquidation_parent_code: "85".parse().unwrap(),
        chart_of_account_interest_income_parent_code: "41".parse().unwrap(),
        chart_of_account_fee_income_parent_code: "42".parse().unwrap(),
        chart_of_account_payment_holding_parent_code: "13".parse().unwrap(),
        chart_of_account_uncovered_outstanding_parent_code: "87".parse().unwrap(),
        chart_of_account_disbursed_defaulted_parent_code: "11".parse().unwrap(),
        chart_of_account_interest_defaulted_parent_code: "11".parse().unwrap(),
        chart_of_account_short_term_individual_disbursed_receivable_parent_code: "11"
            .parse()
            .unwrap(),
        chart_of_account_short_term_government_entity_disbursed_receivable_parent_code: "11"
            .parse()
            .unwrap(),
        chart_of_account_short_term_private_company_disbursed_receivable_parent_code: "11"
            .parse()
            .unwrap(),
        chart_of_account_short_term_bank_disbursed_receivable_parent_code: "11".parse().unwrap(),
        chart_of_account_short_term_financial_institution_disbursed_receivable_parent_code: "11"
            .parse()
            .unwrap(),
        chart_of_account_short_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code:
            "11".parse().unwrap(),
        chart_of_account_short_term_non_domiciled_company_disbursed_receivable_parent_code: "11"
            .parse()
            .unwrap(),
        chart_of_account_long_term_individual_disbursed_receivable_parent_code: "11"
            .parse()
            .unwrap(),
        chart_of_account_long_term_government_entity_disbursed_receivable_parent_code: "11"
            .parse()
            .unwrap(),
        chart_of_account_long_term_private_company_disbursed_receivable_parent_code: "11"
            .parse()
            .unwrap(),
        chart_of_account_long_term_bank_disbursed_receivable_parent_code: "11".parse().unwrap(),
        chart_of_account_long_term_financial_institution_disbursed_receivable_parent_code: "11"
            .parse()
            .unwrap(),
        chart_of_account_long_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code:
            "11".parse().unwrap(),
        chart_of_account_long_term_non_domiciled_company_disbursed_receivable_parent_code: "11"
            .parse()
            .unwrap(),
        chart_of_account_short_term_individual_interest_receivable_parent_code: "12"
            .parse()
            .unwrap(),
        chart_of_account_short_term_government_entity_interest_receivable_parent_code: "12"
            .parse()
            .unwrap(),
        chart_of_account_short_term_private_company_interest_receivable_parent_code: "12"
            .parse()
            .unwrap(),
        chart_of_account_short_term_bank_interest_receivable_parent_code: "12".parse().unwrap(),
        chart_of_account_short_term_financial_institution_interest_receivable_parent_code: "12"
            .parse()
            .unwrap(),
        chart_of_account_short_term_foreign_agency_or_subsidiary_interest_receivable_parent_code:
            "12".parse().unwrap(),
        chart_of_account_short_term_non_domiciled_company_interest_receivable_parent_code: "12"
            .parse()
            .unwrap(),
        chart_of_account_long_term_individual_interest_receivable_parent_code: "12"
            .parse()
            .unwrap(),
        chart_of_account_long_term_government_entity_interest_receivable_parent_code: "12"
            .parse()
            .unwrap(),
        chart_of_account_long_term_private_company_interest_receivable_parent_code: "12"
            .parse()
            .unwrap(),
        chart_of_account_long_term_bank_interest_receivable_parent_code: "12".parse().unwrap(),
        chart_of_account_long_term_financial_institution_interest_receivable_parent_code: "12"
            .parse()
            .unwrap(),
        chart_of_account_long_term_foreign_agency_or_subsidiary_interest_receivable_parent_code:
            "12".parse().unwrap(),
        chart_of_account_long_term_non_domiciled_company_interest_receivable_parent_code: "12"
            .parse()
            .unwrap(),
        chart_of_account_overdue_individual_disbursed_receivable_parent_code: "11".parse().unwrap(),
        chart_of_account_overdue_government_entity_disbursed_receivable_parent_code: "11"
            .parse()
            .unwrap(),
        chart_of_account_overdue_private_company_disbursed_receivable_parent_code: "11"
            .parse()
            .unwrap(),
        chart_of_account_overdue_bank_disbursed_receivable_parent_code: "11".parse().unwrap(),
        chart_of_account_overdue_financial_institution_disbursed_receivable_parent_code: "11"
            .parse()
            .unwrap(),
        chart_of_account_overdue_foreign_agency_or_subsidiary_disbursed_receivable_parent_code:
            "11".parse().unwrap(),
        chart_of_account_overdue_non_domiciled_company_disbursed_receivable_parent_code: "11"
            .parse()
            .unwrap(),
    };
    let res = credit
        .chart_of_accounts_integrations()
        .set_config(
            &DummySubject,
            &chart,
            chart_of_accounts_integration_config.clone(),
        )
        .await
        .unwrap();

    assert_eq!(res, chart_of_accounts_integration_config);

    Ok(())
}
