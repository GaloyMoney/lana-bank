pub mod account;
pub mod account_set;
mod bitfinex;
mod cala;
mod config;
mod constants;
pub mod cursor;
pub mod error;
pub mod loan;
pub mod primitives;
mod tx_template;
pub mod user;

pub use cursor::*;
use tracing::instrument;

use crate::primitives::{
    BfxWithdrawalMethod, LedgerAccountId, LedgerAccountSetId, LedgerTxId, LedgerTxTemplateId,
    LoanId, Satoshis, UsdCents, UserId, WithdrawId,
};

use account_set::{
    LedgerAccountSetAndSubAccounts, LedgerAccountSetAndSubAccountsWithBalance,
    LedgerChartOfAccounts, LedgerTrialBalance, PaginatedLedgerAccountSetSubAccount,
    PaginatedLedgerAccountSetSubAccountWithBalance,
};
use cala::*;
pub use config::*;
use error::*;
use loan::*;
use user::*;

#[derive(Clone)]
pub struct Ledger {
    pub cala: CalaClient,
}

impl Ledger {
    pub async fn init(config: LedgerConfig) -> Result<Self, LedgerError> {
        let cala = CalaClient::new(config.cala_url);
        Self::initialize_tx_templates(&cala).await?;
        Ok(Ledger { cala })
    }

    #[instrument(name = "lava.ledger.get_user_balance", skip(self), err)]
    pub async fn get_user_balance(
        &self,
        account_ids: UserLedgerAccountIds,
    ) -> Result<UserBalance, LedgerError> {
        self.cala
            .get_user_balance(account_ids)
            .await?
            .ok_or(LedgerError::AccountNotFound)
    }

    #[instrument(
        name = "lava.ledger.create_unallocated_collateral_account_for_user",
        skip(self),
        err
    )]
    pub async fn create_accounts_for_user(
        &self,
        user_id: UserId,
    ) -> Result<(UserLedgerAccountIds, UserLedgerAccountAddresses), LedgerError> {
        let account_ids = UserLedgerAccountIds::new();
        let addresses = self.cala.create_user_accounts(user_id, account_ids).await?;
        Ok((account_ids, addresses))
    }

    #[instrument(name = "lava.ledger.add_equity", skip(self), err)]
    pub async fn add_equity(&self, amount: UsdCents, reference: String) -> Result<(), LedgerError> {
        Ok(self
            .cala
            .execute_add_equity_tx(amount.to_usd(), reference)
            .await?)
    }

    #[instrument(name = "lava.ledger.initiate_withdrawal_for_user", skip(self), err)]
    pub async fn initiate_withdrawal_for_user(
        &self,
        withdrawal_id: WithdrawId,
        amount: UsdCents,
        tron_usdt_address: String,
        external_id: String,
        debit_account_id: LedgerAccountId,
    ) -> Result<WithdrawId, LedgerError> {
        Ok(self
            .cala
            .execute_bfx_withdrawal(
                withdrawal_id,
                constants::ON_BALANCE_SHEET_BFX_INTEGRATION_ID.into(),
                amount.to_usd(),
                BfxWithdrawalMethod::TronUsdt,
                tron_usdt_address,
                debit_account_id,
                external_id,
            )
            .await?)
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
        tx_id: LedgerTxId,
        loan_account_ids: LoanAccountIds,
        user_account_ids: UserLedgerAccountIds,
        collateral: Satoshis,
        principal: UsdCents,
        external_id: String,
    ) -> Result<(), LedgerError> {
        Ok(self
            .cala
            .execute_approve_loan_tx(
                tx_id,
                loan_account_ids,
                user_account_ids,
                collateral.to_btc(),
                principal.to_usd(),
                external_id,
            )
            .await?)
    }

    #[instrument(name = "lava.ledger.record_interest", skip(self), err)]
    pub async fn record_loan_interest(
        &self,
        tx_id: LedgerTxId,
        loan_account_ids: LoanAccountIds,
        tx_ref: String,
        amount: UsdCents,
    ) -> Result<(), LedgerError> {
        Ok(self
            .cala
            .execute_incur_interest_tx(tx_id, loan_account_ids, amount.to_usd(), tx_ref)
            .await?)
    }

    #[instrument(name = "lava.ledger.record_payment", skip(self), err)]
    pub async fn record_payment(
        &self,
        tx_id: LedgerTxId,
        loan_account_ids: LoanAccountIds,
        user_account_ids: UserLedgerAccountIds,
        amount: UsdCents,
        tx_ref: String,
    ) -> Result<(), LedgerError> {
        Ok(self
            .cala
            .execute_repay_loan_tx(
                tx_id,
                loan_account_ids,
                user_account_ids,
                amount.to_usd(),
                tx_ref,
            )
            .await?)
    }

    #[instrument(name = "lava.ledger.complete_loan", skip(self), err)]
    pub async fn complete_loan(
        &self,
        tx_id: LedgerTxId,
        loan_account_ids: LoanAccountIds,
        user_account_ids: UserLedgerAccountIds,
        payment_amount: UsdCents,
        collateral_amount: Satoshis,
        tx_ref: String,
    ) -> Result<(), LedgerError> {
        Ok(self
            .cala
            .execute_complete_loan_tx(
                tx_id,
                loan_account_ids,
                user_account_ids,
                payment_amount.to_usd(),
                collateral_amount.to_btc(),
                tx_ref,
            )
            .await?)
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

    pub async fn trial_balance(&self) -> Result<Option<LedgerTrialBalance>, LedgerError> {
        self.cala
            .trial_balance::<LedgerTrialBalance>()
            .await
            .map(|gl| gl.map(LedgerTrialBalance::from))
            .map_err(|e| e.into())
    }

    pub async fn obs_trial_balance(&self) -> Result<Option<LedgerTrialBalance>, LedgerError> {
        self.cala
            .obs_trial_balance::<LedgerTrialBalance>()
            .await
            .map(|gl| gl.map(LedgerTrialBalance::from))
            .map_err(|e| e.into())
    }

    pub async fn chart_of_accounts(&self) -> Result<Option<LedgerChartOfAccounts>, LedgerError> {
        self.cala
            .chart_of_accounts::<LedgerChartOfAccounts>()
            .await
            .map(|gl| gl.map(LedgerChartOfAccounts::from))
            .map_err(|e| e.into())
    }

    pub async fn obs_chart_of_accounts(
        &self,
    ) -> Result<Option<LedgerChartOfAccounts>, LedgerError> {
        self.cala
            .obs_chart_of_accounts::<LedgerChartOfAccounts>()
            .await
            .map(|gl| gl.map(LedgerChartOfAccounts::from))
            .map_err(|e| e.into())
    }

    pub async fn account_set_and_sub_accounts(
        &self,
        account_set_id: LedgerAccountSetId,
        first: i64,
        after: Option<String>,
    ) -> Result<Option<LedgerAccountSetAndSubAccounts>, LedgerError> {
        self.cala
            .find_account_set_and_sub_accounts_by_id::<LedgerAccountSetAndSubAccounts>(
                account_set_id,
                first,
                after,
            )
            .await
            .map(|gl| gl.map(LedgerAccountSetAndSubAccounts::from))
            .map_err(|e| e.into())
    }

    pub async fn paginated_account_set_and_sub_accounts(
        &self,
        account_set_id: LedgerAccountSetId,
        query: crate::query::PaginatedQueryArgs<SubAccountCursor>,
    ) -> Result<
        crate::query::PaginatedQueryRet<PaginatedLedgerAccountSetSubAccount, SubAccountCursor>,
        LedgerError,
    > {
        let account_set = self
            .cala
            .find_account_set_and_sub_accounts_by_id::<LedgerAccountSetAndSubAccounts>(
                account_set_id,
                i64::try_from(query.first)?,
                query.after.map(|c| c.value),
            )
            .await
            .map(|gl| gl.map(LedgerAccountSetAndSubAccounts::from))
            .map_err(LedgerError::from)?;

        let (sub_accounts, has_next_page, end_cursor) =
            account_set.map_or((Vec::new(), false, None), |account_set| {
                (
                    account_set.sub_accounts.members,
                    account_set.sub_accounts.page_info.has_next_page,
                    account_set
                        .sub_accounts
                        .page_info
                        .end_cursor
                        .map(|end_cursor| SubAccountCursor { value: end_cursor }),
                )
            });

        Ok(crate::query::PaginatedQueryRet {
            entities: sub_accounts,
            has_next_page,
            end_cursor,
        })
    }

    pub async fn account_set_and_sub_accounts_with_balance(
        &self,
        account_set_id: LedgerAccountSetId,
        first: i64,
        after: Option<String>,
    ) -> Result<Option<LedgerAccountSetAndSubAccountsWithBalance>, LedgerError> {
        self.cala
            .find_account_set_and_sub_accounts_with_balance_by_id::<LedgerAccountSetAndSubAccountsWithBalance>(
                account_set_id,
                first,
                after,
            )
            .await
            .map(|gl| gl.map(LedgerAccountSetAndSubAccountsWithBalance::from))
            .map_err(|e| e.into())
    }

    pub async fn paginated_account_set_and_sub_accounts_with_balance(
        &self,
        account_set_id: LedgerAccountSetId,
        query: crate::query::PaginatedQueryArgs<SubAccountCursor>,
    ) -> Result<
        crate::query::PaginatedQueryRet<
            PaginatedLedgerAccountSetSubAccountWithBalance,
            SubAccountCursor,
        >,
        LedgerError,
    > {
        let account_set = self
            .cala
            .find_account_set_and_sub_accounts_with_balance_by_id::<LedgerAccountSetAndSubAccountsWithBalance>(
                account_set_id,
                i64::try_from(query.first)?,
                query.after.map(|c| c.value),
            )
            .await
            .map(|gl| gl.map(LedgerAccountSetAndSubAccountsWithBalance::from))
            .map_err(LedgerError::from)?;

        let (sub_accounts, has_next_page, end_cursor) =
            account_set.map_or((Vec::new(), false, None), |account_set| {
                (
                    account_set.sub_accounts.members,
                    account_set.sub_accounts.page_info.has_next_page,
                    account_set
                        .sub_accounts
                        .page_info
                        .end_cursor
                        .map(|end_cursor| SubAccountCursor { value: end_cursor }),
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

        Self::assert_approve_loan_tx_template_exists(cala, constants::APPROVE_LOAN_CODE).await?;

        Self::assert_incur_interest_tx_template_exists(cala, constants::INCUR_INTEREST_CODE)
            .await?;

        Self::assert_record_payment_tx_template_exists(cala, constants::RECORD_PAYMENT_CODE)
            .await?;

        Self::assert_complete_loan_tx_template_exists(cala, constants::COMPLETE_LOAN_CODE).await?;

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

    async fn assert_approve_loan_tx_template_exists(
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
        let err = match cala.create_approve_loan_tx_template(template_id).await {
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

    async fn assert_complete_loan_tx_template_exists(
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
        let err = match cala.create_complete_loan_tx_template(template_id).await {
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
