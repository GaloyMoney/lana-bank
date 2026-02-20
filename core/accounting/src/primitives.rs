use rust_decimal::Decimal;

// Re-export everything from core-accounting-types
pub use core_accounting_types::*;

// Additional re-exports that depend on cala-ledger (not in core-types)
pub use cala_ledger::{
    account::Account as CalaAccount,
    account_set::AccountSet as CalaAccountSet,
    balance::{AccountBalance as CalaAccountBalance, BalanceRange as CalaBalanceRange},
};

#[derive(Debug, Clone)]
pub struct BalanceRange {
    pub open: Option<CalaAccountBalance>,
    pub close: Option<CalaAccountBalance>,
    pub period_activity: Option<CalaAccountBalance>,
}

impl BalanceRange {
    pub(crate) fn has_non_zero_activity(&self) -> bool {
        if let Some(close) = self.close.as_ref() {
            close.details.settled.dr_balance != Decimal::ZERO
                || close.details.settled.cr_balance != Decimal::ZERO
                || close.details.pending.dr_balance != Decimal::ZERO
                || close.details.pending.cr_balance != Decimal::ZERO
                || close.details.encumbrance.dr_balance != Decimal::ZERO
                || close.details.encumbrance.cr_balance != Decimal::ZERO
        } else {
            false
        }
    }
}

// Implement ChartLookup for our Chart entity
impl ChartLookup for crate::chart_of_accounts::Chart {
    fn id(&self) -> ChartId {
        self.id
    }

    fn find_account_set_id_in_category(
        &self,
        code: &AccountCode,
        category: AccountCategory,
    ) -> Option<CalaAccountSetId> {
        // Delegate to the Chart entity's existing method
        self.find_account_set_id_in_category(code, category)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chart_level() {
        let parent = "11".parse::<AccountCodeSection>().unwrap();
        let sub = "01".parse::<AccountCodeSection>().unwrap();
        let child = "0201".parse::<AccountCodeSection>().unwrap();

        let account_code = AccountCode::new(vec![parent.clone()]);
        assert_eq!(account_code.chart_level(), 0);

        let account_code = AccountCode::new(vec![parent.clone(), sub.clone()]);
        assert_eq!(account_code.chart_level(), 1);

        let account_code = AccountCode::new(vec![parent, sub, child]);
        assert_eq!(account_code.chart_level(), 2);
    }

    #[test]
    fn is_equivalent_to_str() {
        let parent = "11".parse::<AccountCodeSection>().unwrap();
        let sub = "01".parse::<AccountCodeSection>().unwrap();
        let child = "0201".parse::<AccountCodeSection>().unwrap();

        let account_code = AccountCode::new(vec![parent, sub, child]);
        assert!(account_code.is_equivalent_to_str("11010201"));
        assert!(!account_code.is_equivalent_to_str("110102010"));
    }

    #[test]
    fn errors_for_new_spec_if_invalid_parent() {
        let parent = "10".parse::<AccountCode>().unwrap();
        let child = "11".parse::<AccountCode>().unwrap();
        let new_spec = AccountSpec::try_new(
            Some(parent),
            child.sections().to_vec(),
            "spec".parse().unwrap(),
            Default::default(),
        );
        assert!(matches!(new_spec, Err(AccountCodeError::InvalidParent)));
    }

    mod is_parent_of {
        use super::*;

        #[test]
        fn not_parent_when_child_sections_empty() {
            let parent = "10".parse::<AccountCode>().unwrap();
            let child = AccountCode::new(vec![]);
            assert!(!parent.is_parent_of(child.sections()));
        }

        #[test]
        fn not_parent_when_parent_sections_empty() {
            let parent = AccountCode::new(vec![]);
            let child = "10".parse::<AccountCode>().unwrap();
            assert!(!parent.is_parent_of(child.sections()));
        }

        #[test]
        fn is_parent_when_prefix_matches_in_first_section() {
            let parent = "1".parse::<AccountCode>().unwrap();
            let child = "11".parse::<AccountCode>().unwrap();
            assert!(parent.is_parent_of(child.sections()));
        }

        #[test]
        fn not_parent_when_prefix_does_not_match_in_first_section() {
            let parent = "10".parse::<AccountCode>().unwrap();
            let child = "11".parse::<AccountCode>().unwrap();
            assert!(!parent.is_parent_of(child.sections()));
        }

        #[test]
        fn is_parent_when_child_has_more_sections_than_parent() {
            let parent = "10".parse::<AccountCode>().unwrap();
            let child = "10.20".parse::<AccountCode>().unwrap();
            assert!(parent.is_parent_of(child.sections()));

            let parent = "10.20".parse::<AccountCode>().unwrap();
            let child = "10.20.0201".parse::<AccountCode>().unwrap();
            assert!(parent.is_parent_of(child.sections()));
        }

        #[test]
        fn not_parent_when_child_has_more_sections_than_parent() {
            let parent = "10.20".parse::<AccountCode>().unwrap();
            let child = "10".parse::<AccountCode>().unwrap();
            assert!(!parent.is_parent_of(child.sections()));
        }

        #[test]
        fn not_parent_when_sections_equal() {
            let parent = "10".parse::<AccountCode>().unwrap();
            let child = "10".parse::<AccountCode>().unwrap();
            assert!(!parent.is_parent_of(child.sections()));
        }

        #[test]
        fn not_parent_when_parent_code_longer_but_prefixed() {
            let parent = "100".parse::<AccountCode>().unwrap();
            let child = "10".parse::<AccountCode>().unwrap();
            assert!(!parent.is_parent_of(child.sections()));
        }

        #[test]
        fn not_parent_when_parent_code_longer_but_prefixed_in_second_section() {
            let parent = "1.23".parse::<AccountCode>().unwrap();
            let child = "1.2".parse::<AccountCode>().unwrap();
            assert!(!parent.is_parent_of(child.sections()));
        }

        #[test]
        fn not_parent_when_prefix_mismatch_in_second_section() {
            let parent = "1.23".parse::<AccountCode>().unwrap();
            let child = "1.20".parse::<AccountCode>().unwrap();
            assert!(!parent.is_parent_of(child.sections()));
        }
    }

    mod check_valid_parent {
        use super::*;

        #[test]
        fn ok_when_no_parent() {
            let child = "10.20".parse::<AccountCode>().unwrap();
            assert!(child.check_valid_parent(None).is_ok());
        }

        #[test]
        fn ok_when_is_parent() {
            let parent = "1".parse::<AccountCode>().unwrap();
            let child = "11".parse::<AccountCode>().unwrap();
            assert!(child.check_valid_parent(Some(parent)).is_ok());
        }

        #[test]
        fn err_when_not_parent() {
            let parent = "10".parse::<AccountCode>().unwrap();
            let child = "11".parse::<AccountCode>().unwrap();
            assert!(matches!(
                child.check_valid_parent(Some(parent)),
                Err(AccountCodeError::InvalidParent)
            ));
        }
    }

    mod accounting_base_config {
        use super::*;

        fn default_config() -> AccountingBaseConfig {
            AccountingBaseConfig::try_new(
                "1".parse().unwrap(),
                "2".parse().unwrap(),
                "3".parse().unwrap(),
                "32.01".parse().unwrap(),
                "32.02".parse().unwrap(),
                "4".parse().unwrap(),
                "5".parse().unwrap(),
                "6".parse().unwrap(),
            )
            .unwrap()
        }

        #[test]
        fn try_new_ok_with_valid_config() {
            let config = default_config();

            assert!(config.assets_code.is_top_level_chart_code());
            assert!(
                config
                    .equity_code
                    .is_parent_of(config.equity_retained_earnings_gain_code.sections())
            );
        }

        #[test]
        fn try_new_err_when_invalid_config_dup_code() {
            let invalid_config_res = AccountingBaseConfig::try_new(
                "1".parse().unwrap(),
                "1".parse().unwrap(),
                "3".parse().unwrap(),
                "32.01".parse().unwrap(),
                "32.02".parse().unwrap(),
                "4".parse().unwrap(),
                "5".parse().unwrap(),
                "6".parse().unwrap(),
            );
            assert!(matches!(
                invalid_config_res,
                Err(AccountingBaseConfigError::DuplicateAccountCode(_))
            ))
        }

        #[test]
        fn try_new_err_when_invalid_config_not_top_level() {
            let invalid_config_res = AccountingBaseConfig::try_new(
                "11".parse().unwrap(),
                "2".parse().unwrap(),
                "3".parse().unwrap(),
                "32.01".parse().unwrap(),
                "32.02".parse().unwrap(),
                "4".parse().unwrap(),
                "5".parse().unwrap(),
                "6".parse().unwrap(),
            );
            assert!(matches!(
                invalid_config_res,
                Err(AccountingBaseConfigError::AccountCodeNotTopLevel(_))
            ))
        }

        #[test]
        fn try_new_err_when_invalid_config_retained_earnings_not_child_of_equity() {
            let invalid_config_res = AccountingBaseConfig::try_new(
                "1".parse().unwrap(),
                "2".parse().unwrap(),
                "3".parse().unwrap(),
                "92.01".parse().unwrap(),
                "92.02".parse().unwrap(),
                "4".parse().unwrap(),
                "5".parse().unwrap(),
                "6".parse().unwrap(),
            );
            assert!(matches!(
                invalid_config_res,
                Err(AccountingBaseConfigError::RetainedEarningsCodeNotChildOfEquity(_))
            ))
        }

        #[test]
        fn is_off_balance_sheet_returns_false_for_configured_codes() {
            let config = default_config();

            assert!(!config.is_off_balance_sheet_account_set_or_account(&config.assets_code));
            assert!(!config.is_off_balance_sheet_account_set_or_account(&config.liabilities_code));
            assert!(!config.is_off_balance_sheet_account_set_or_account(&config.equity_code));
            assert!(!config.is_off_balance_sheet_account_set_or_account(&config.revenue_code));
            assert!(
                !config.is_off_balance_sheet_account_set_or_account(&config.cost_of_revenue_code)
            );
            assert!(!config.is_off_balance_sheet_account_set_or_account(&config.expenses_code));
            assert!(!config.is_off_balance_sheet_account_set_or_account(
                &config.equity_retained_earnings_gain_code
            ));
            assert!(!config.is_off_balance_sheet_account_set_or_account(
                &config.equity_retained_earnings_loss_code
            ));
        }

        #[test]
        fn is_off_balance_sheet_returns_true_for_non_configured_top_level_codes() {
            let config = default_config();
            let code = "9".parse::<AccountCode>().unwrap();
            assert!(config.is_off_balance_sheet_account_set_or_account(&code));
        }

        #[test]
        fn is_off_balance_sheet_returns_true_for_non_configured_child_codes() {
            let config = default_config();
            let code = "91".parse::<AccountCode>().unwrap();
            assert!(config.is_off_balance_sheet_account_set_or_account(&code));
        }

        #[test]
        fn is_assets_returns_true_for_top_level_asset_code() {
            let config = default_config();
            assert!(config.is_assets_account_set_or_account(&config.assets_code));
        }

        #[test]
        fn is_assets_returns_true_for_child_account_set_member() {
            let config = default_config();
            let top_chart_level_account_code = "11".parse::<AccountCode>().unwrap();
            let child_account_code = "11.1".parse::<AccountCode>().unwrap();

            assert!(config.is_assets_account_set_or_account(&top_chart_level_account_code));
            assert!(config.is_assets_account_set_or_account(&child_account_code));
        }

        #[test]
        fn is_assets_returns_false_for_non_asset_code() {
            let config = default_config();
            let off_balance_sheet_code = "9".parse::<AccountCode>().unwrap();
            assert!(!config.is_assets_account_set_or_account(&off_balance_sheet_code));
            assert!(!config.is_assets_account_set_or_account(&config.equity_code));
        }

        #[test]
        fn is_account_in_category_delegates_correctly() {
            let config = default_config();
            let off_balance_sheet_code = "9".parse::<AccountCode>().unwrap();
            let asset_child_code = "11".parse::<AccountCode>().unwrap();

            // OffBalanceSheet
            assert!(
                config.is_account_in_category(
                    &off_balance_sheet_code,
                    AccountCategory::OffBalanceSheet
                )
            );
            assert!(
                !config
                    .is_account_in_category(&config.assets_code, AccountCategory::OffBalanceSheet)
            );

            // Asset
            assert!(config.is_account_in_category(&config.assets_code, AccountCategory::Asset));
            assert!(config.is_account_in_category(&asset_child_code, AccountCategory::Asset));
            assert!(
                !config.is_account_in_category(&config.liabilities_code, AccountCategory::Asset)
            );

            // Liability
            assert!(
                config.is_account_in_category(&config.liabilities_code, AccountCategory::Liability)
            );
            assert!(
                !config.is_account_in_category(&config.assets_code, AccountCategory::Liability)
            );

            // Equity
            assert!(config.is_account_in_category(&config.equity_code, AccountCategory::Equity));
            assert!(config.is_account_in_category(
                &config.equity_retained_earnings_gain_code,
                AccountCategory::Equity
            ));
            assert!(!config.is_account_in_category(&config.assets_code, AccountCategory::Equity));

            // Revenue
            assert!(config.is_account_in_category(&config.revenue_code, AccountCategory::Revenue));
            assert!(!config.is_account_in_category(&config.assets_code, AccountCategory::Revenue));
        }
    }
}
