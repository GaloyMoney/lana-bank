pub mod account;
pub mod account_set;
pub mod bank;
mod cala;
mod config;
mod constants;
pub mod credit_facility;
pub mod customer;
pub mod disbursal;
pub mod error;
pub mod primitives;

use authz::PermissionCheck;

use chrono::{DateTime, Utc};
use disbursal::DisbursalData;
use tracing::instrument;

use crate::{
    authorization::{Authorization, LedgerAction, Object},
    primitives::{
        CollateralAction, CreditFacilityId, CustomerId, LedgerAccountSetId, LedgerTxTemplateId,
        Subject, UsdCents,
    },
};

use account_set::{
    LedgerAccountSetAndSubAccountsWithBalance, LedgerBalanceSheet, LedgerCashFlowStatement,
    LedgerChartOfAccounts, LedgerProfitAndLossStatement, LedgerSubAccountCursor,
    LedgerTrialBalance, PaginatedLedgerAccountSetSubAccountWithBalance,
};
use bank::*;
use cala::*;
pub use config::*;
use credit_facility::*;
use customer::*;
use error::*;

#[derive(Clone)]
pub struct Ledger {
    cala: CalaClient,
    authz: Authorization,
}

impl Ledger {
    pub async fn init(config: LedgerConfig, authz: &Authorization) -> Result<Self, LedgerError> {
        let cala = CalaClient::new(config.cala_url);
        Self::initialize_tx_templates(&cala).await?;
        Ok(Ledger {
            cala,
            authz: authz.clone(),
        })
    }

    #[instrument(name = "lana.ledger.get_bank_deposits_balance", skip(self), err)]
    async fn get_bank_deposits_balance(&self) -> Result<BankDepositsBalance, LedgerError> {
        self.cala
            .find_account_with_balance_by_code::<BankDepositsBalance, LedgerError>(
                constants::BANK_DEPOSITS_OMNIBUS_CODE.into(),
            )
            .await?
            .ok_or(LedgerError::AccountNotFound)
    }

    #[instrument(name = "lana.ledger.get_customer_balance", skip(self), err)]
    pub async fn get_customer_balance(
        &self,
        account_ids: CustomerLedgerAccountIds,
    ) -> Result<CustomerBalance, LedgerError> {
        self.cala
            .get_customer_balance(account_ids)
            .await?
            .ok_or(LedgerError::AccountNotFound)
    }

    #[instrument(
        name = "lana.ledger.create_unallocated_collateral_account_for_customer",
        skip(self),
        err
    )]
    pub async fn create_accounts_for_customer(
        &self,
        customer_id: CustomerId,
    ) -> Result<CustomerLedgerAccountIds, LedgerError> {
        let account_ids = CustomerLedgerAccountIds::new();
        self.cala
            .create_customer_accounts(customer_id, account_ids)
            .await?;
        Ok(account_ids)
    }

    #[instrument(name = "lana.ledger.add_equity", skip(self), err)]
    pub async fn add_equity(&self, amount: UsdCents, reference: String) -> Result<(), LedgerError> {
        Ok(self
            .cala
            .execute_add_equity_tx(amount.to_usd(), reference)
            .await?)
    }

    #[instrument(name = "lana.ledger.credit_facility_balance", skip(self), err)]
    pub async fn get_credit_facility_balance(
        &self,
        account_ids: CreditFacilityAccountIds,
    ) -> Result<CreditFacilityLedgerBalance, LedgerError> {
        self.cala
            .get_credit_facility_balance(account_ids)
            .await?
            .ok_or(LedgerError::AccountNotFound)
    }

    #[instrument(
        name = "lana.ledger.record_credit_facility_interest_incurrence",
        skip(self),
        err
    )]
    pub async fn record_credit_facility_interest_incurrence(
        &self,
        CreditFacilityInterestIncurrence {
            interest,
            period,
            tx_ref,
            tx_id,
            credit_facility_account_ids,
        }: CreditFacilityInterestIncurrence,
    ) -> Result<chrono::DateTime<chrono::Utc>, LedgerError> {
        Ok(self
            .cala
            .execute_credit_facility_incur_interest_tx(
                tx_id,
                credit_facility_account_ids,
                interest.to_usd(),
                tx_ref,
                period.end,
            )
            .await?)
    }

    #[instrument(name = "lana.ledger.activate_credit_facility", skip(self), err)]
    pub async fn activate_credit_facility(
        &self,
        CreditFacilityActivationData {
            tx_id,
            tx_ref,
            credit_facility_account_ids,
            customer_account_ids,
            facility,
            structuring_fee,
        }: CreditFacilityActivationData,
    ) -> Result<chrono::DateTime<chrono::Utc>, LedgerError> {
        Ok(self
            .cala
            .execute_approve_credit_facility_tx(
                tx_id,
                credit_facility_account_ids,
                customer_account_ids,
                facility.to_usd(),
                structuring_fee.to_usd(),
                tx_ref,
            )
            .await?)
    }

    #[instrument(
        name = "lana.ledger.record_credit_facility_interest_accrual",
        skip(self),
        err
    )]
    pub async fn record_credit_facility_interest_accrual(
        &self,
        CreditFacilityInterestAccrual {
            interest,
            tx_ref,
            tx_id,
            accrued_at,
            credit_facility_account_ids,
        }: CreditFacilityInterestAccrual,
    ) -> Result<chrono::DateTime<chrono::Utc>, LedgerError> {
        Ok(self
            .cala
            .execute_credit_facility_accrue_interest_tx(
                tx_id,
                credit_facility_account_ids,
                interest.to_usd(),
                tx_ref,
                accrued_at,
            )
            .await?)
    }

    #[instrument(name = "lana.ledger.record_disbursal", skip(self), err)]
    pub async fn record_disbursal(
        &self,
        DisbursalData {
            amount,
            tx_ref,
            tx_id,
            account_ids,
            customer_account_ids,
        }: DisbursalData,
    ) -> Result<chrono::DateTime<chrono::Utc>, LedgerError> {
        Ok(self
            .cala
            .execute_credit_facility_disbursal_tx(
                tx_id,
                account_ids,
                customer_account_ids,
                amount.to_usd(),
                tx_ref,
            )
            .await?)
    }

    #[instrument(
        name = "lana.ledger.manage_credit_facility_collateral",
        skip(self),
        err
    )]
    pub async fn update_credit_facility_collateral(
        &self,
        CreditFacilityCollateralUpdate {
            tx_id,
            credit_facility_account_ids,
            abs_diff,
            tx_ref,
            action,
        }: CreditFacilityCollateralUpdate,
    ) -> Result<chrono::DateTime<chrono::Utc>, LedgerError> {
        let created_at = match action {
            CollateralAction::Add => {
                self.cala
                    .add_credit_facility_collateral(
                        tx_id,
                        credit_facility_account_ids,
                        abs_diff.to_btc(),
                        tx_ref,
                    )
                    .await
            }
            CollateralAction::Remove => {
                self.cala
                    .remove_credit_facility_collateral(
                        tx_id,
                        credit_facility_account_ids,
                        abs_diff.to_btc(),
                        tx_ref,
                    )
                    .await
            }
        }?;
        Ok(created_at)
    }

    #[instrument(name = "lana.ledger.record_credit_facility_repayment", skip(self), err)]
    pub async fn record_credit_facility_repayment(
        &self,
        CreditFacilityRepayment {
            tx_id,
            tx_ref,
            credit_facility_account_ids,
            customer_account_ids,
            amounts:
                CreditFacilityPaymentAmounts {
                    interest,
                    disbursal,
                },
        }: CreditFacilityRepayment,
    ) -> Result<chrono::DateTime<chrono::Utc>, LedgerError> {
        Ok(self
            .cala
            .execute_repay_credit_facility_tx(
                tx_id,
                credit_facility_account_ids,
                customer_account_ids,
                interest.to_usd(),
                disbursal.to_usd(),
                tx_ref,
            )
            .await?)
    }

    #[instrument(name = "lana.ledger.complete_credit_facility", skip(self), err)]
    pub async fn complete_credit_facility(
        &self,
        CreditFacilityCompletion {
            tx_id,
            tx_ref,
            collateral,
            credit_facility_account_ids,
            customer_account_ids,
        }: CreditFacilityCompletion,
    ) -> Result<chrono::DateTime<chrono::Utc>, LedgerError> {
        let created_at = self
            .cala
            .remove_credit_facility_collateral(
                tx_id,
                credit_facility_account_ids,
                collateral.to_btc(),
                tx_ref,
            )
            .await?;
        Ok(created_at)
    }

    #[instrument(
        name = "lana.ledger.create_accounts_for_credit_facility",
        skip(self),
        err
    )]
    pub async fn create_accounts_for_credit_facility(
        &self,
        credit_facility_id: CreditFacilityId,
        credit_facility_account_ids: CreditFacilityAccountIds,
    ) -> Result<(), LedgerError> {
        self.cala
            .create_credit_facility_accounts(credit_facility_id, credit_facility_account_ids)
            .await?;
        Ok(())
    }

    pub async fn trial_balance(
        &self,
        sub: &Subject,
        from: DateTime<Utc>,
        until: Option<DateTime<Utc>>,
    ) -> Result<Option<LedgerTrialBalance>, LedgerError> {
        self.authz
            .enforce_permission(sub, Object::Ledger, LedgerAction::Read)
            .await?;
        self.cala
            .trial_balance::<LedgerTrialBalance, LedgerError>(from, until)
            .await
    }

    pub async fn obs_trial_balance(
        &self,
        sub: &Subject,
        from: DateTime<Utc>,
        until: Option<DateTime<Utc>>,
    ) -> Result<Option<LedgerTrialBalance>, LedgerError> {
        self.authz
            .enforce_permission(sub, Object::Ledger, LedgerAction::Read)
            .await?;
        self.cala
            .obs_trial_balance::<LedgerTrialBalance, LedgerError>(from, until)
            .await
    }

    pub async fn chart_of_accounts(
        &self,
        sub: &Subject,
    ) -> Result<Option<LedgerChartOfAccounts>, LedgerError> {
        self.authz
            .enforce_permission(sub, Object::Ledger, LedgerAction::Read)
            .await?;
        self.cala
            .chart_of_accounts::<LedgerChartOfAccounts, LedgerError>()
            .await
    }

    pub async fn obs_chart_of_accounts(
        &self,
        sub: &Subject,
    ) -> Result<Option<LedgerChartOfAccounts>, LedgerError> {
        self.authz
            .enforce_permission(sub, Object::Ledger, LedgerAction::Read)
            .await?;
        Ok(self
            .cala
            .obs_chart_of_accounts::<LedgerChartOfAccounts, LedgerError>()
            .await?
            .map(LedgerChartOfAccounts::from))
    }

    pub async fn balance_sheet(
        &self,
        sub: &Subject,
        from: DateTime<Utc>,
        until: Option<DateTime<Utc>>,
    ) -> Result<Option<LedgerBalanceSheet>, LedgerError> {
        self.authz
            .enforce_permission(sub, Object::Ledger, LedgerAction::Read)
            .await?;
        Ok(self
            .cala
            .balance_sheet::<LedgerBalanceSheet, LedgerError>(from, until)
            .await?
            .map(LedgerBalanceSheet::from))
    }

    pub async fn profit_and_loss(
        &self,
        sub: &Subject,
        from: DateTime<Utc>,
        until: Option<DateTime<Utc>>,
    ) -> Result<Option<LedgerProfitAndLossStatement>, LedgerError> {
        self.authz
            .enforce_permission(sub, Object::Ledger, LedgerAction::Read)
            .await?;
        Ok(self
            .cala
            .profit_and_loss::<LedgerProfitAndLossStatement, LedgerError>(from, until)
            .await?
            .map(LedgerProfitAndLossStatement::from))
    }

    pub async fn cash_flow(
        &self,
        sub: &Subject,
        from: DateTime<Utc>,
        until: Option<DateTime<Utc>>,
    ) -> Result<Option<LedgerCashFlowStatement>, LedgerError> {
        self.authz
            .enforce_permission(sub, Object::Ledger, LedgerAction::Read)
            .await?;
        Ok(self
            .cala
            .cash_flow::<LedgerCashFlowStatement, LedgerError>(from, until)
            .await?
            .map(LedgerCashFlowStatement::from))
    }

    pub async fn account_set_and_sub_accounts_with_balance(
        &self,
        sub: &Subject,
        account_set_id: LedgerAccountSetId,
        first: i64,
        after: Option<String>,
        from: DateTime<Utc>,
        until: Option<DateTime<Utc>>,
    ) -> Result<Option<LedgerAccountSetAndSubAccountsWithBalance>, LedgerError> {
        self.authz
            .enforce_permission(sub, Object::Ledger, LedgerAction::Read)
            .await?;
        Ok(self.cala
            .find_account_set_and_sub_accounts_with_balance_by_id::<LedgerAccountSetAndSubAccountsWithBalance, LedgerError>(
                account_set_id,
                first,
                after,from, until
            )
            .await?
            .map(LedgerAccountSetAndSubAccountsWithBalance::from))
    }

    pub async fn paginated_account_set_and_sub_accounts_with_balance(
        &self,
        account_set_id: LedgerAccountSetId,
        from: DateTime<Utc>,
        until: Option<DateTime<Utc>>,
        query: es_entity::PaginatedQueryArgs<LedgerSubAccountCursor>,
    ) -> Result<
        es_entity::PaginatedQueryRet<
            PaginatedLedgerAccountSetSubAccountWithBalance,
            LedgerSubAccountCursor,
        >,
        LedgerError,
    > {
        let account_set = self
            .cala
            .find_account_set_and_sub_accounts_with_balance_by_id::<LedgerAccountSetAndSubAccountsWithBalance, LedgerError>(
                account_set_id,
                i64::try_from(query.first)?,
                query.after.map(|c| c.value),
                from, until
            )
            .await?
            .map(LedgerAccountSetAndSubAccountsWithBalance::from);

        let (sub_accounts, has_next_page, end_cursor) =
            account_set.map_or((Vec::new(), false, None), |account_set| {
                (
                    account_set.sub_accounts.members,
                    account_set.sub_accounts.page_info.has_next_page,
                    account_set
                        .sub_accounts
                        .page_info
                        .end_cursor
                        .map(|end_cursor| LedgerSubAccountCursor { value: end_cursor }),
                )
            });

        Ok(es_entity::PaginatedQueryRet {
            entities: sub_accounts,
            has_next_page,
            end_cursor,
        })
    }

    async fn initialize_tx_templates(cala: &CalaClient) -> Result<(), LedgerError> {
        Self::assert_add_equity_tx_template_exists(cala, constants::ADD_EQUITY_CODE).await?;
        Self::assert_deposit_template_tx_template_exists(cala, constants::DEPOSIT_CHECKING).await?;
        Self::assert_initiate_withdraw_template_tx_template_exists(
            cala,
            constants::INITIATE_WITHDRAW,
        )
        .await?;
        Self::assert_confirm_withdraw_tx_template_exists(cala, constants::CONFIRM_WITHDRAW).await?;

        Self::assert_cancel_withdraw_tx_template_exists(cala, constants::CANCEL_WITHDRAW).await?;
        Self::assert_approve_credit_facility_tx_template_exists(
            cala,
            constants::APPROVE_CREDIT_FACILITY_CODE,
        )
        .await?;
        Self::assert_credit_facility_disbursal_tx_template_exists(
            cala,
            constants::CREDIT_FACILITY_DISBURSAL_CODE,
        )
        .await?;
        Self::assert_add_collateral_tx_template_exists(cala, constants::ADD_COLLATERAL_CODE)
            .await?;
        Self::assert_remove_collateral_tx_template_exists(cala, constants::REMOVE_COLLATERAL_CODE)
            .await?;
        Self::assert_incur_interest_tx_template_exists(cala, constants::INCUR_INTEREST_CODE)
            .await?;
        Self::assert_credit_facility_incur_interest_tx_template_exists(
            cala,
            constants::CREDIT_FACILITY_INCUR_INTEREST_CODE,
        )
        .await?;
        Self::assert_credit_facility_accrue_interest_tx_template_exists(
            cala,
            constants::CREDIT_FACILITY_ACCRUE_INTEREST_CODE,
        )
        .await?;

        Self::assert_record_payment_tx_template_exists(cala, constants::RECORD_PAYMENT_CODE)
            .await?;
        Self::assert_record_credit_facility_payment_tx_template_exists(
            cala,
            constants::RECORD_CREDIT_FACILITY_PAYMENT_CODE,
        )
        .await?;

        Ok(())
    }

    async fn assert_add_equity_tx_template_exists(
        cala: &CalaClient,
        template_code: &str,
    ) -> Result<LedgerTxTemplateId, LedgerError> {
        if let Ok(id) = cala
            .find_tx_template_by_code::<LedgerTxTemplateId>(template_code.to_owned())
            .await
        {
            return Ok(id);
        }

        let template_id = LedgerTxTemplateId::new();
        let err = match cala.create_add_equity_tx_template(template_id).await {
            Ok(id) => {
                return Ok(id);
            }
            Err(e) => e,
        };

        Ok(cala
            .find_tx_template_by_code::<LedgerTxTemplateId>(template_code.to_owned())
            .await
            .map_err(|_| err)?)
    }

    async fn assert_deposit_template_tx_template_exists(
        cala: &CalaClient,
        template_code: &str,
    ) -> Result<LedgerTxTemplateId, LedgerError> {
        if let Ok(id) = cala
            .find_tx_template_by_code::<LedgerTxTemplateId>(template_code.to_owned())
            .await
        {
            return Ok(id);
        }

        let template_id = LedgerTxTemplateId::new();
        let err = match cala.create_deposit_checking_tx_template(template_id).await {
            Ok(id) => {
                return Ok(id);
            }
            Err(e) => e,
        };

        Ok(cala
            .find_tx_template_by_code::<LedgerTxTemplateId>(template_code.to_owned())
            .await
            .map_err(|_| err)?)
    }

    async fn assert_initiate_withdraw_template_tx_template_exists(
        cala: &CalaClient,
        template_code: &str,
    ) -> Result<LedgerTxTemplateId, LedgerError> {
        if let Ok(id) = cala
            .find_tx_template_by_code::<LedgerTxTemplateId>(template_code.to_owned())
            .await
        {
            return Ok(id);
        }

        let template_id = LedgerTxTemplateId::new();
        let err = match cala.create_initiate_withdraw_tx_template(template_id).await {
            Ok(id) => {
                return Ok(id);
            }
            Err(e) => e,
        };

        Ok(cala
            .find_tx_template_by_code::<LedgerTxTemplateId>(template_code.to_owned())
            .await
            .map_err(|_| err)?)
    }

    async fn assert_confirm_withdraw_tx_template_exists(
        cala: &CalaClient,
        template_code: &str,
    ) -> Result<LedgerTxTemplateId, LedgerError> {
        if let Ok(id) = cala
            .find_tx_template_by_code::<LedgerTxTemplateId>(template_code.to_owned())
            .await
        {
            return Ok(id);
        }

        let template_id = LedgerTxTemplateId::new();
        let err = match cala.create_confirm_withdraw_tx_template(template_id).await {
            Ok(id) => {
                return Ok(id);
            }
            Err(e) => e,
        };

        Ok(cala
            .find_tx_template_by_code::<LedgerTxTemplateId>(template_code.to_owned())
            .await
            .map_err(|_| err)?)
    }

    async fn assert_cancel_withdraw_tx_template_exists(
        cala: &CalaClient,
        template_code: &str,
    ) -> Result<LedgerTxTemplateId, LedgerError> {
        if let Ok(id) = cala
            .find_tx_template_by_code::<LedgerTxTemplateId>(template_code.to_owned())
            .await
        {
            return Ok(id);
        }

        let template_id = LedgerTxTemplateId::new();
        let err = match cala.create_cancel_withdraw_tx_template(template_id).await {
            Ok(id) => {
                return Ok(id);
            }
            Err(e) => e,
        };

        Ok(cala
            .find_tx_template_by_code::<LedgerTxTemplateId>(template_code.to_owned())
            .await
            .map_err(|_| err)?)
    }

    async fn assert_approve_credit_facility_tx_template_exists(
        cala: &CalaClient,
        template_code: &str,
    ) -> Result<LedgerTxTemplateId, LedgerError> {
        if let Ok(id) = cala
            .find_tx_template_by_code::<LedgerTxTemplateId>(template_code.to_owned())
            .await
        {
            return Ok(id);
        }

        let template_id = LedgerTxTemplateId::new();
        let err = match cala
            .create_approve_credit_facility_tx_template(template_id)
            .await
        {
            Ok(id) => {
                return Ok(id);
            }
            Err(e) => e,
        };

        Ok(cala
            .find_tx_template_by_code::<LedgerTxTemplateId>(template_code.to_owned())
            .await
            .map_err(|_| err)?)
    }

    async fn assert_add_collateral_tx_template_exists(
        cala: &CalaClient,
        template_code: &str,
    ) -> Result<LedgerTxTemplateId, LedgerError> {
        if let Ok(id) = cala
            .find_tx_template_by_code::<LedgerTxTemplateId>(template_code.to_owned())
            .await
        {
            return Ok(id);
        }

        let template_id = LedgerTxTemplateId::new();
        let err = match cala.create_add_collateral_tx_template(template_id).await {
            Ok(id) => {
                return Ok(id);
            }
            Err(e) => e,
        };

        Ok(cala
            .find_tx_template_by_code::<LedgerTxTemplateId>(template_code.to_owned())
            .await
            .map_err(|_| err)?)
    }

    async fn assert_remove_collateral_tx_template_exists(
        cala: &CalaClient,
        template_code: &str,
    ) -> Result<LedgerTxTemplateId, LedgerError> {
        if let Ok(id) = cala
            .find_tx_template_by_code::<LedgerTxTemplateId>(template_code.to_owned())
            .await
        {
            return Ok(id);
        }

        let template_id = LedgerTxTemplateId::new();
        let err = match cala.create_remove_collateral_tx_template(template_id).await {
            Ok(id) => {
                return Ok(id);
            }
            Err(e) => e,
        };

        Ok(cala
            .find_tx_template_by_code::<LedgerTxTemplateId>(template_code.to_owned())
            .await
            .map_err(|_| err)?)
    }

    async fn assert_credit_facility_disbursal_tx_template_exists(
        cala: &CalaClient,
        template_code: &str,
    ) -> Result<LedgerTxTemplateId, LedgerError> {
        if let Ok(id) = cala
            .find_tx_template_by_code::<LedgerTxTemplateId>(template_code.to_owned())
            .await
        {
            return Ok(id);
        }

        let template_id = LedgerTxTemplateId::new();
        let err = match cala
            .create_facility_disbursal_tx_template(template_id)
            .await
        {
            Ok(id) => {
                return Ok(id);
            }
            Err(e) => e,
        };

        Ok(cala
            .find_tx_template_by_code::<LedgerTxTemplateId>(template_code.to_owned())
            .await
            .map_err(|_| err)?)
    }

    async fn assert_incur_interest_tx_template_exists(
        cala: &CalaClient,
        template_code: &str,
    ) -> Result<LedgerTxTemplateId, LedgerError> {
        if let Ok(id) = cala
            .find_tx_template_by_code::<LedgerTxTemplateId>(template_code.to_owned())
            .await
        {
            return Ok(id);
        }

        let template_id = LedgerTxTemplateId::new();
        let err = match cala.create_incur_interest_tx_template(template_id).await {
            Ok(id) => {
                return Ok(id);
            }
            Err(e) => e,
        };

        Ok(cala
            .find_tx_template_by_code::<LedgerTxTemplateId>(template_code.to_owned())
            .await
            .map_err(|_| err)?)
    }

    async fn assert_credit_facility_incur_interest_tx_template_exists(
        cala: &CalaClient,
        template_code: &str,
    ) -> Result<LedgerTxTemplateId, LedgerError> {
        if let Ok(id) = cala
            .find_tx_template_by_code::<LedgerTxTemplateId>(template_code.to_owned())
            .await
        {
            return Ok(id);
        }

        let template_id = LedgerTxTemplateId::new();
        let err = match cala
            .create_credit_facility_incur_interest_tx_template(template_id)
            .await
        {
            Ok(id) => {
                return Ok(id);
            }
            Err(e) => e,
        };

        Ok(cala
            .find_tx_template_by_code::<LedgerTxTemplateId>(template_code.to_owned())
            .await
            .map_err(|_| err)?)
    }

    async fn assert_credit_facility_accrue_interest_tx_template_exists(
        cala: &CalaClient,
        template_code: &str,
    ) -> Result<LedgerTxTemplateId, LedgerError> {
        if let Ok(id) = cala
            .find_tx_template_by_code::<LedgerTxTemplateId>(template_code.to_owned())
            .await
        {
            return Ok(id);
        }

        let template_id = LedgerTxTemplateId::new();
        let err = match cala
            .create_credit_facility_accrue_interest_tx_template(template_id)
            .await
        {
            Ok(id) => {
                return Ok(id);
            }
            Err(e) => e,
        };

        Ok(cala
            .find_tx_template_by_code::<LedgerTxTemplateId>(template_code.to_owned())
            .await
            .map_err(|_| err)?)
    }

    async fn assert_record_payment_tx_template_exists(
        cala: &CalaClient,
        template_code: &str,
    ) -> Result<LedgerTxTemplateId, LedgerError> {
        if let Ok(id) = cala
            .find_tx_template_by_code::<LedgerTxTemplateId>(template_code.to_owned())
            .await
        {
            return Ok(id);
        }

        let template_id = LedgerTxTemplateId::new();
        let err = match cala.create_record_payment_tx_template(template_id).await {
            Ok(id) => {
                return Ok(id);
            }
            Err(e) => e,
        };

        Ok(cala
            .find_tx_template_by_code::<LedgerTxTemplateId>(template_code.to_owned())
            .await
            .map_err(|_| err)?)
    }

    async fn assert_record_credit_facility_payment_tx_template_exists(
        cala: &CalaClient,
        template_code: &str,
    ) -> Result<LedgerTxTemplateId, LedgerError> {
        if let Ok(id) = cala
            .find_tx_template_by_code::<LedgerTxTemplateId>(template_code.to_owned())
            .await
        {
            return Ok(id);
        }

        let template_id = LedgerTxTemplateId::new();
        let err = match cala
            .create_record_credit_facility_payment_tx_template(template_id)
            .await
        {
            Ok(id) => {
                return Ok(id);
            }
            Err(e) => e,
        };

        Ok(cala
            .find_tx_template_by_code::<LedgerTxTemplateId>(template_code.to_owned())
            .await
            .map_err(|_| err)?)
    }
}
