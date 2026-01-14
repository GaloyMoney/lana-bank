mod helpers;

use rand::Rng;

use authz::dummy::DummySubject;
use cala_ledger::{CalaLedger, CalaLedgerConfig};
use cloud_storage::{Storage, config::StorageConfig};
use domain_config::DomainConfigs;

use core_accounting::{AccountingBaseConfig, CoreAccounting};
use core_credit::*;
use document_storage::DocumentStorage;
use helpers::{action, event, object};
use public_id::PublicIds;

#[tokio::test]
async fn chart_of_accounts_integration() -> anyhow::Result<()> {
    let pool = helpers::init_pool().await?;

    let outbox =
        obix::Outbox::<event::DummyEvent>::init(&pool, obix::MailboxConfig::default()).await?;
    let authz = authz::dummy::DummyPerms::<action::DummyAction, object::DummyObject>::new();
    let domain_configs = DomainConfigs::new(&pool);
    let storage = Storage::new(&StorageConfig::default());
    let document_storage = DocumentStorage::new(&pool, &storage);

    let governance = governance::Governance::new(&pool, &authz, &outbox);
    let public_ids = public_id::PublicIds::new(&pool);
    let customers =
        core_customer::Customers::new(&pool, &authz, &outbox, document_storage, public_ids);
    let custody =
        core_custody::CoreCustody::init(&pool, &authz, helpers::custody_config(), &outbox).await?;

    let cala_config = CalaLedgerConfig::builder()
        .pool(pool.clone())
        .exec_migrations(false)
        .build()?;
    let cala = CalaLedger::init(cala_config).await?;
    let jobs = job::Jobs::init(
        job::JobSvcConfig::builder()
            .pool(pool.clone())
            .build()
            .unwrap(),
    )
    .await?;

    let mut job_new = job_new::Jobs::init(
        job_new::JobSvcConfig::builder()
            .pool(pool.clone())
            .build()
            .unwrap(),
    )
    .await?;

    let journal_id = helpers::init_journal(&cala).await?;
    let public_ids = PublicIds::new(&pool);
    let price = core_price::Price::init(&mut job_new, &outbox).await?;

    let credit = CoreCredit::init(
        &pool,
        Default::default(),
        &governance,
        &jobs,
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

    let accounting_document_storage = DocumentStorage::new(&pool, &storage);
    let accounting = CoreAccounting::new(
        &pool,
        &authz,
        &cala,
        journal_id,
        accounting_document_storage,
        &mut job_new,
        &domain_configs,
    );
    let chart_ref = format!("ref-{:08}", rand::rng().random_range(0..10000));
    let chart_id = accounting
        .chart_of_accounts()
        .create_chart(&DummySubject, "Test chart".to_string(), chart_ref.clone())
        .await?
        .id;
    let import = r#"
    1,,,Assets,Debit,
    11,,,Receivables,,
    12,,,Payment Holding,,
    2,,,Liabilities,Credit,
    3,,,Equity,Credit,
    32,,,Retained Earnings,,
    ,01,,Current Year Earnings,,
    ,02,,Prior Years Earnings,,
    4,,,Revenue,Credit,
    41,,,Interest Income,,
    42,,,Fee Income,,
    5,,,Cost of Revenue,Debit,
    6,,,Expenses,Debit,
    8,,,Memorandum,Debit,
    81,,,Facility Omnibus,,
    82,,,Collateral Omnibus,,
    83,,,Facility,,
    84,,,Collateral,,
    85,,,Collateral in Liquidation,,
    86,,,Liquidation Proceeds Omnibus,,
    "#
    .to_string();
    let base_config = AccountingBaseConfig {
        assets_code: "1".parse().unwrap(),
        liabilities_code: "2".parse().unwrap(),
        equity_code: "3".parse().unwrap(),
        equity_retained_earnings_gain_code: "32.01".parse().unwrap(),
        equity_retained_earnings_loss_code: "32.02".parse().unwrap(),
        revenue_code: "4".parse().unwrap(),
        cost_of_revenue_code: "5".parse().unwrap(),
        expenses_code: "6".parse().unwrap(),
    };
    let (chart, _) = accounting
        .chart_of_accounts()
        .import_from_csv_with_base_config(&DummySubject, &chart_ref, import, base_config.clone())
        .await?;

    let code = "1".parse::<core_accounting::AccountCode>().unwrap();
    let account_set_id = cala
        .account_sets()
        .find(chart.account_set_id_from_code(&code).unwrap())
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
                chart_of_account_liquidation_proceeds_omnibus_parent_code: "83".parse().unwrap(),
                chart_of_account_facility_parent_code: "81".parse().unwrap(),
                chart_of_account_collateral_parent_code: "82".parse().unwrap(),
                chart_of_account_collateral_in_liquidation_parent_code: "83".parse().unwrap(),
                chart_of_account_interest_income_parent_code: "41".parse().unwrap(),
                chart_of_account_fee_income_parent_code: "42".parse().unwrap(),
                chart_of_account_payment_holding_parent_code: "12".parse().unwrap(),
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
                chart_of_account_short_term_individual_interest_receivable_parent_code: "11".parse().unwrap(),
                chart_of_account_short_term_government_entity_interest_receivable_parent_code:
                    "11".parse().unwrap(),
                chart_of_account_short_term_private_company_interest_receivable_parent_code:
                    "11".parse().unwrap(),
                chart_of_account_short_term_bank_interest_receivable_parent_code: "11".parse().unwrap(),
                chart_of_account_short_term_financial_institution_interest_receivable_parent_code:
                    "11".parse().unwrap(),
                chart_of_account_short_term_foreign_agency_or_subsidiary_interest_receivable_parent_code:
                    "11".parse().unwrap(),
                chart_of_account_short_term_non_domiciled_company_interest_receivable_parent_code:
                    "11".parse().unwrap(),
                chart_of_account_long_term_individual_interest_receivable_parent_code: "11".parse().unwrap(),
                chart_of_account_long_term_government_entity_interest_receivable_parent_code:
                    "11".parse().unwrap(),
                chart_of_account_long_term_private_company_interest_receivable_parent_code:
                    "11".parse().unwrap(),
                chart_of_account_long_term_bank_interest_receivable_parent_code: "11".parse().unwrap(),
                chart_of_account_long_term_financial_institution_interest_receivable_parent_code:
                    "11".parse().unwrap(),
                chart_of_account_long_term_foreign_agency_or_subsidiary_interest_receivable_parent_code:
                    "11".parse().unwrap(),
                chart_of_account_long_term_non_domiciled_company_interest_receivable_parent_code:
                    "11".parse().unwrap(),
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

    let res = cala
        .account_sets()
        .list_members_by_created_at(account_set_id, Default::default())
        .await?;

    assert_eq!(res.entities.len(), 2);

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

    let import = r#"
        1,Other Facility Omnibus Parent
        2,Other Collateral Omnibus Parent
        3,Other Facility Parent
        4,Other Collateral Parent
        5,Other Disbursed Receivable Parent
        6,Other Interest Receivable Parent
        7,Other Interest Income Parent
        8,Other Fee Income Parent
        9,Other Payment Holding Parent
        "#
    .to_string();
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
                chart_of_account_facility_omnibus_parent_code: "1".parse().unwrap(),
                chart_of_account_collateral_omnibus_parent_code: "2".parse().unwrap(),
                chart_of_account_liquidation_proceeds_omnibus_parent_code: "1".parse().unwrap(),
                chart_of_account_facility_parent_code: "3".parse().unwrap(),
                chart_of_account_collateral_parent_code: "4".parse().unwrap(),
                chart_of_account_collateral_in_liquidation_parent_code: "3".parse().unwrap(),
                chart_of_account_interest_income_parent_code: "7".parse().unwrap(),
                chart_of_account_fee_income_parent_code: "8".parse().unwrap(),
                chart_of_account_payment_holding_parent_code: "9".parse().unwrap(),
                chart_of_account_short_term_individual_disbursed_receivable_parent_code: "1".parse().unwrap(),
                chart_of_account_short_term_government_entity_disbursed_receivable_parent_code:
                    "2".parse().unwrap(),
                chart_of_account_short_term_private_company_disbursed_receivable_parent_code:
                    "3".parse().unwrap(),
                chart_of_account_short_term_bank_disbursed_receivable_parent_code: "4".parse().unwrap(),
                chart_of_account_short_term_financial_institution_disbursed_receivable_parent_code:
                    "5".parse().unwrap(),
                chart_of_account_short_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code:
                    "6".parse().unwrap(),
                chart_of_account_short_term_non_domiciled_company_disbursed_receivable_parent_code:
                    "7".parse().unwrap(),
                chart_of_account_long_term_individual_disbursed_receivable_parent_code: "1".parse().unwrap(),
                chart_of_account_long_term_government_entity_disbursed_receivable_parent_code:
                    "2".parse().unwrap(),
                chart_of_account_long_term_private_company_disbursed_receivable_parent_code:
                    "3".parse().unwrap(),
                chart_of_account_long_term_bank_disbursed_receivable_parent_code: "4".parse().unwrap(),
                chart_of_account_long_term_financial_institution_disbursed_receivable_parent_code:
                    "5".parse().unwrap(),
                chart_of_account_long_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code:
                    "6".parse().unwrap(),
                chart_of_account_long_term_non_domiciled_company_disbursed_receivable_parent_code:
                    "7".parse().unwrap(),
                chart_of_account_short_term_individual_interest_receivable_parent_code: "1".parse().unwrap(),
                chart_of_account_short_term_government_entity_interest_receivable_parent_code:
                    "2".parse().unwrap(),
                chart_of_account_short_term_private_company_interest_receivable_parent_code:
                    "3".parse().unwrap(),
                chart_of_account_short_term_bank_interest_receivable_parent_code: "4".parse().unwrap(),
                chart_of_account_short_term_financial_institution_interest_receivable_parent_code:
                    "5".parse().unwrap(),
                chart_of_account_short_term_foreign_agency_or_subsidiary_interest_receivable_parent_code:
                    "6".parse().unwrap(),
                chart_of_account_short_term_non_domiciled_company_interest_receivable_parent_code:
                    "7".parse().unwrap(),
                chart_of_account_long_term_individual_interest_receivable_parent_code: "1".parse().unwrap(),
                chart_of_account_long_term_government_entity_interest_receivable_parent_code:
                    "2".parse().unwrap(),
                chart_of_account_long_term_private_company_interest_receivable_parent_code:
                    "3".parse().unwrap(),
                chart_of_account_long_term_bank_interest_receivable_parent_code: "4".parse().unwrap(),
                chart_of_account_long_term_financial_institution_interest_receivable_parent_code:
                    "5".parse().unwrap(),
                chart_of_account_long_term_foreign_agency_or_subsidiary_interest_receivable_parent_code:
                    "6".parse().unwrap(),
                chart_of_account_long_term_non_domiciled_company_interest_receivable_parent_code:
                    "7".parse().unwrap(),
                chart_of_account_overdue_individual_disbursed_receivable_parent_code: "1".parse().unwrap(),
                chart_of_account_overdue_government_entity_disbursed_receivable_parent_code:
                    "2".parse().unwrap(),
                chart_of_account_overdue_private_company_disbursed_receivable_parent_code:
                    "3".parse().unwrap(),
                chart_of_account_overdue_bank_disbursed_receivable_parent_code: "4".parse().unwrap(),
                chart_of_account_overdue_financial_institution_disbursed_receivable_parent_code:
                    "5".parse().unwrap(),
                chart_of_account_overdue_foreign_agency_or_subsidiary_disbursed_receivable_parent_code:
                    "6".parse().unwrap(),
                chart_of_account_overdue_non_domiciled_company_disbursed_receivable_parent_code:
                    "7".parse().unwrap(),
            },
        )
        .await;

    assert!(matches!(
        res,
        Err(core_credit::ChartOfAccountsIntegrationError::CreditConfigAlreadyExists)
    ));

    Ok(())
}
