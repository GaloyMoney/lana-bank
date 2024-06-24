mod account;
mod account_ledger;
mod account_set;
mod bitfinex;
mod cala;
mod config;
mod constants;
pub mod error;
pub mod fixed_term_loan;
pub mod primitives;
mod tx_template;
pub mod user;

use tracing::instrument;

use crate::primitives::{
    BfxWithdrawalMethod, FixedTermLoanId, LedgerAccountId, LedgerTxId, LedgerTxTemplateId,
    Satoshis, UsdCents, UserId, WithdrawId,
};

use account_ledger::*;
use cala::*;
pub use config::*;
use error::*;
use fixed_term_loan::*;
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

    #[instrument(name = "lava.ledger.get_fixed_term_loan_balance", skip(self), err)]
    pub async fn get_fixed_term_loan_balance(
        &self,
        account_ids: FixedTermLoanAccountIds,
    ) -> Result<FixedTermLoanBalance, LedgerError> {
        self.cala
            .get_fixed_term_loan_balance(account_ids)
            .await?
            .ok_or(LedgerError::AccountNotFound)
    }

    #[instrument(name = "lava.ledger.approve_loan", skip(self), err)]
    pub async fn approve_loan(
        &self,
        tx_id: LedgerTxId,
        loan_account_ids: FixedTermLoanAccountIds,
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
    pub async fn record_interest(
        &self,
        tx_id: LedgerTxId,
        loan_account_ids: FixedTermLoanAccountIds,
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
        loan_account_ids: FixedTermLoanAccountIds,
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
        loan_account_ids: FixedTermLoanAccountIds,
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

    #[instrument(
        name = "lava.ledger.create_unallocated_collateral_account_for_user",
        skip(self),
        err
    )]
    pub async fn create_accounts_for_loan(
        &self,
        loan_id: FixedTermLoanId,
        loan_account_ids: FixedTermLoanAccountIds,
    ) -> Result<(), LedgerError> {
        self.cala
            .create_loan_accounts(loan_id, loan_account_ids)
            .await?;
        Ok(())
    }

    pub async fn account_general_ledger_summary(
        &self,
    ) -> Result<Option<AccountLedgerSummary>, LedgerError> {
        match self.cala.general_ledger::<AccountLedgerSummary>().await {
            Ok(gl) => Ok(gl),
            Err(e) => Err(e)?,
        }
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
