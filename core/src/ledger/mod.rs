pub mod account;
pub mod account_set;
pub mod bank;
mod cala;
mod config;
mod constants;
pub mod customer;
pub mod error;
pub mod loan;
pub mod primitives;

use chrono::{DateTime, Utc};
use tracing::instrument;

use crate::{
    authorization::{Authorization, LedgerAction, Object},
    primitives::{
        CollateralAction, CustomerId, DepositId, LedgerAccountId, LedgerAccountSetId, LedgerTxId,
        LedgerTxTemplateId, LoanId, Subject, UsdCents, WithdrawId,
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
use customer::*;
use error::*;
use loan::*;

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

    #[instrument(name = "lava.ledger.get_bank_deposits_balance", skip(self), err)]
    async fn get_bank_deposits_balance(&self) -> Result<BankDepositsBalance, LedgerError> {
        self.cala
            .find_account_with_balance_by_code::<BankDepositsBalance, LedgerError>(
                constants::BANK_DEPOSITS_OMNIBUS_CODE.into(),
            )
            .await?
            .ok_or(LedgerError::AccountNotFound)
    }

    #[instrument(name = "lava.ledger.get_customer_balance", skip(self), err)]
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
        name = "lava.ledger.create_unallocated_collateral_account_for_customer",
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

    #[instrument(name = "lava.ledger.add_equity", skip(self), err)]
    pub async fn add_equity(&self, amount: UsdCents, reference: String) -> Result<(), LedgerError> {
        Ok(self
            .cala
            .execute_add_equity_tx(amount.to_usd(), reference)
            .await?)
    }

    #[instrument(name = "lava.ledger.record_deposit_for_customer", skip(self), err)]
    pub async fn record_deposit_for_customer(
        &self,
        deposit_id: DepositId,
        customer_account_ids: CustomerLedgerAccountIds,
        amount: UsdCents,
        external_id: String,
    ) -> Result<DepositId, LedgerError> {
        self.cala
            .execute_deposit_checking_tx(
                LedgerTxId::from(uuid::Uuid::from(deposit_id)),
                customer_account_ids,
                amount.to_usd(),
                external_id,
            )
            .await?;
        Ok(deposit_id)
    }

    #[instrument(name = "lava.ledger.initiate_withdrawal_for_customer", skip(self), err)]
    pub async fn initiate_withdrawal_for_customer(
        &self,
        withdrawal_id: WithdrawId,
        customer_account_ids: CustomerLedgerAccountIds,
        amount: UsdCents,
        external_id: String,
    ) -> Result<WithdrawId, LedgerError> {
        self.get_bank_deposits_balance()
            .await?
            .check_withdrawal_amount(amount)?;

        self.cala
            .execute_initiate_withdraw_tx(
                LedgerTxId::from(uuid::Uuid::from(withdrawal_id)),
                customer_account_ids,
                amount.to_usd(),
                external_id,
            )
            .await?;
        Ok(withdrawal_id)
    }

    #[instrument(name = "lava.ledger.confirm_withdrawal_for_customer", skip(self), err)]
    pub async fn confirm_withdrawal_for_customer(
        &self,
        ledger_tx_id: LedgerTxId,
        withdrawal_id: WithdrawId,
        debit_account_id: LedgerAccountId,
        amount: UsdCents,
        external_id: String,
    ) -> Result<WithdrawId, LedgerError> {
        self.cala
            .execute_confirm_withdraw_tx(
                ledger_tx_id,
                uuid::Uuid::from(withdrawal_id),
                debit_account_id,
                amount.to_usd(),
                external_id,
            )
            .await?;
        Ok(withdrawal_id)
    }

    #[instrument(name = "lava.ledger.cancel_withdrawal_for_customer", skip(self), err)]
    pub async fn cancel_withdrawal_for_customer(
        &self,
        ledger_tx_id: LedgerTxId,
        withdrawal_id: WithdrawId,
        debit_account_id: LedgerAccountId,
        amount: UsdCents,
        external_id: String,
    ) -> Result<WithdrawId, LedgerError> {
        self.cala
            .execute_cancel_withdraw_tx(
                ledger_tx_id,
                uuid::Uuid::from(withdrawal_id),
                debit_account_id,
                amount.to_usd(),
                external_id,
            )
            .await?;
        Ok(withdrawal_id)
    }

    #[instrument(name = "lava.ledger.loan_balance", skip(self), err)]
    pub async fn get_loan_balance(
        &self,
        account_ids: LoanAccountIds,
    ) -> Result<LoanBalance, LedgerError> {
        self.cala
            .get_loan_balance(account_ids)
            .await?
            .ok_or(LedgerError::AccountNotFound)
    }

    #[instrument(name = "lava.ledger.approve_loan", skip(self), err)]
    pub async fn approve_loan(
        &self,
        LoanApprovalData {
            tx_id,
            tx_ref,
            loan_account_ids,
            customer_account_ids,
            initial_principal,
        }: LoanApprovalData,
    ) -> Result<chrono::DateTime<chrono::Utc>, LedgerError> {
        Ok(self
            .cala
            .execute_approve_loan_facility_tx(
                tx_id,
                loan_account_ids,
                initial_principal.to_usd(),
                tx_ref,
            )
            .await?)
    }

    #[instrument(name = "lava.ledger.record_interest", skip(self), err)]
    pub async fn record_loan_interest(
        &self,
        LoanInterestAccrual {
            interest,
            tx_ref,
            tx_id,
            loan_account_ids,
        }: LoanInterestAccrual,
    ) -> Result<chrono::DateTime<chrono::Utc>, LedgerError> {
        Ok(self
            .cala
            .execute_incur_interest_tx(tx_id, loan_account_ids, interest.to_usd(), tx_ref)
            .await?)
    }

    #[instrument(name = "lava.ledger.record_loan_payment", skip(self), err)]
    pub async fn record_loan_repayment(
        &self,
        repayment: LoanRepayment,
    ) -> Result<DateTime<Utc>, LedgerError> {
        let executed_at = match repayment {
            LoanRepayment::Partial {
                tx_id,
                tx_ref,
                loan_account_ids,
                customer_account_ids,
                amounts:
                    LoanPaymentAmounts {
                        interest,
                        principal,
                    },
            } => {
                self.cala
                    .execute_repay_loan_tx(
                        tx_id,
                        loan_account_ids,
                        customer_account_ids,
                        interest.to_usd(),
                        principal.to_usd(),
                        tx_ref,
                    )
                    .await?
            }
            LoanRepayment::Final {
                payment_tx_id,
                payment_tx_ref,
                collateral_tx_id,
                collateral_tx_ref,
                collateral,
                loan_account_ids,
                customer_account_ids,
                amounts:
                    LoanPaymentAmounts {
                        interest,
                        principal,
                    },
            } => {
                self.cala
                    .execute_complete_loan_tx(
                        payment_tx_id,
                        collateral_tx_id,
                        loan_account_ids,
                        customer_account_ids,
                        interest.to_usd(),
                        principal.to_usd(),
                        collateral.to_btc(),
                        payment_tx_ref,
                        collateral_tx_ref,
                    )
                    .await?
            }
        };
        Ok(executed_at)
    }

    #[instrument(name = "lava.ledger.manage_collateral", skip(self), err)]
    pub async fn update_collateral(
        &self,
        LoanCollateralUpdate {
            tx_id,
            loan_account_ids,
            abs_diff,
            tx_ref,
            action,
        }: LoanCollateralUpdate,
    ) -> Result<chrono::DateTime<chrono::Utc>, LedgerError> {
        let created_at = match action {
            CollateralAction::Add => {
                self.cala
                    .add_collateral(tx_id, loan_account_ids, abs_diff.to_btc(), tx_ref)
                    .await
            }
            CollateralAction::Remove => {
                self.cala
                    .remove_collateral(tx_id, loan_account_ids, abs_diff.to_btc(), tx_ref)
                    .await
            }
        }?;
        Ok(created_at)
    }

    #[instrument(name = "lava.ledger.create_accounts_for_loan", skip(self), err)]
    pub async fn create_accounts_for_loan(
        &self,
        loan_id: LoanId,
        loan_account_ids: LoanAccountIds,
    ) -> Result<(), LedgerError> {
        self.cala
            .create_loan_accounts(loan_id, loan_account_ids)
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
            .check_permission(sub, Object::Ledger, LedgerAction::Read)
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
            .check_permission(sub, Object::Ledger, LedgerAction::Read)
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
            .check_permission(sub, Object::Ledger, LedgerAction::Read)
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
            .check_permission(sub, Object::Ledger, LedgerAction::Read)
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
            .check_permission(sub, Object::Ledger, LedgerAction::Read)
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
            .check_permission(sub, Object::Ledger, LedgerAction::Read)
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
            .check_permission(sub, Object::Ledger, LedgerAction::Read)
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
            .check_permission(sub, Object::Ledger, LedgerAction::Read)
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
        query: crate::query::PaginatedQueryArgs<LedgerSubAccountCursor>,
    ) -> Result<
        crate::query::PaginatedQueryRet<
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

        Ok(crate::query::PaginatedQueryRet {
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
        Self::assert_approve_loan_facility_tx_template_exists(
            cala,
            constants::APPROVE_LOAN_FACILITY_CODE,
        )
        .await?;
        Self::assert_loan_disbursement_tx_template_exists(
            cala,
            constants::INCREMENTAL_PRINCIPAL_DISBURSEMENT_CODE,
        )
        .await?;
        Self::assert_add_collateral_tx_template_exists(cala, constants::ADD_COLLATERAL_CODE)
            .await?;
        Self::assert_remove_collateral_tx_template_exists(cala, constants::REMOVE_COLLATERAL_CODE)
            .await?;
        Self::assert_incur_interest_tx_template_exists(cala, constants::INCUR_INTEREST_CODE)
            .await?;

        Self::assert_record_payment_tx_template_exists(cala, constants::RECORD_PAYMENT_CODE)
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

    async fn assert_approve_loan_facility_tx_template_exists(
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
            .create_approve_loan_facility_tx_template(template_id)
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

    async fn assert_loan_disbursement_tx_template_exists(
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
        let err = match cala.create_loan_disbursement_tx_template(template_id).await {
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
}
