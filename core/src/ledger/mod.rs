pub mod bank;
mod cala;
mod config;
mod constants;
pub mod error;
pub mod fixed_term_loan;
pub mod primitives;
mod tx_template;
pub mod user;

use bank::BankLedgerAccountIds;
use tracing::instrument;

use crate::primitives::{
    BankId, FixedTermLoanId, LedgerAccountId, LedgerDebitOrCredit, LedgerTxId, LedgerTxTemplateId,
    Satoshis, UsdCents,
};

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
        Self::initialize_journal(&cala).await?;
        Self::initialize_global_accounts(&cala).await?;
        Self::initialize_tx_templates(&cala).await?;
        Ok(Ledger { cala })
    }

    pub fn default_bank_id(&self) -> BankId {
        BankId::from(constants::BANK_ID)
    }

    #[instrument(name = "lava.ledger.create_accounts_for_bank", skip(self), err)]
    pub async fn create_accounts_for_bank(&self) -> Result<BankLedgerAccountIds, LedgerError> {
        let default_bank_account_ids = BankLedgerAccountIds::default();

        Self::assert_credit_account_exists(
            &self.cala,
            default_bank_account_ids.shareholder_equity_id,
            constants::BANK_SHAREHOLDER_EQUITY_NAME,
            constants::BANK_SHAREHOLDER_EQUITY_CODE,
            &default_bank_account_ids.shareholder_equity_id.to_string(),
        )
        .await?;

        Ok(default_bank_account_ids)
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

    #[instrument(name = "lava.ledger.create_accounts_for_user", skip(self), err)]
    pub async fn create_accounts_for_user(
        &self,
        bitfinex_username: &str,
    ) -> Result<UserLedgerAccountIds, LedgerError> {
        let account_ids = UserLedgerAccountIds::new();
        Self::assert_credit_account_exists(
            &self.cala,
            account_ids.unallocated_collateral_id,
            &format!("USERS.UNALLOCATED_COLLATERAL.{}", bitfinex_username),
            &format!("USERS.UNALLOCATED_COLLATERAL.{}", bitfinex_username),
            &format!("usr:bfx-{}:unallocated_collateral", bitfinex_username),
        )
        .await?;

        Self::assert_credit_account_exists(
            &self.cala,
            account_ids.checking_id,
            &format!("USERS.CHECKING.{}", bitfinex_username),
            &format!("USERS.CHECKING.{}", bitfinex_username),
            &format!("usr:bfx-{}:checking", bitfinex_username),
        )
        .await?;

        Ok(account_ids)
    }

    #[instrument(name = "lava.ledger.deposit_checking_for_user", skip(self), err)]
    pub async fn pledge_collateral_for_user(
        &self,
        id: LedgerAccountId,
        amount: Satoshis,
        reference: String,
    ) -> Result<(), LedgerError> {
        Ok(self
            .cala
            .execute_pledge_unallocated_collateral_tx(id, amount.to_btc(), reference)
            .await?)
    }

    #[instrument(name = "lava.ledger.deposit_checking_for_user", skip(self), err)]
    pub async fn deposit_checking_for_user(
        &self,
        id: LedgerAccountId,
        amount: UsdCents,
        reference: String,
    ) -> Result<(), LedgerError> {
        Ok(self
            .cala
            .execute_deposit_checking_tx(id, amount.to_usd(), reference)
            .await?)
    }

    #[instrument(name = "lava.ledger.initiate_withdrawal_for_user", skip(self), err)]
    pub async fn initiate_withdrawal_for_user(
        &self,
        user_account_ids: UserLedgerAccountIds,
        amount: UsdCents,
        reference: String,
    ) -> Result<(), LedgerError> {
        Ok(self
            .cala
            .execute_initiate_withdrawal_from_checking_tx(
                user_account_ids,
                amount.to_usd(),
                reference,
            )
            .await?)
    }

    #[instrument(name = "lava.ledger.settle_withdrawal_for_user", skip(self), err)]
    pub async fn settle_withdrawal_for_user(
        &self,
        user_account_ids: UserLedgerAccountIds,
        amount: UsdCents,
        reference: String,
    ) -> Result<(), LedgerError> {
        Ok(self
            .cala
            .execute_settle_withdrawal_from_checking_tx(
                user_account_ids,
                amount.to_usd(),
                reference,
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
        FixedTermLoanAccountIds {
            collateral_account_id,
            outstanding_account_id,
            interest_income_account_id,
        }: FixedTermLoanAccountIds,
    ) -> Result<(), LedgerError> {
        Self::assert_credit_account_exists(
            &self.cala,
            collateral_account_id,
            &format!("LOAN.COLLATERAL.{}", loan_id),
            &format!("LOAN.COLLATERAL.{}", loan_id),
            &format!("LOAN.COLLATERAL.{}", loan_id),
        )
        .await?;

        Self::assert_debit_account_exists(
            &self.cala,
            outstanding_account_id,
            &format!("LOAN.OUTSTANDING.{}", loan_id),
            &format!("LOAN.OUTSTANDING.{}", loan_id),
            &format!("LOAN.OUTSTANDING.{}", loan_id),
        )
        .await?;

        Self::assert_credit_account_exists(
            &self.cala,
            interest_income_account_id,
            &format!("LOAN.INTEREST_INCOME.{}", loan_id),
            &format!("LOAN.INTEREST_INCOME.{}", loan_id),
            &format!("LOAN.INTEREST_INCOME.{}", loan_id),
        )
        .await?;

        Ok(())
    }

    async fn initialize_journal(cala: &CalaClient) -> Result<(), LedgerError> {
        if cala
            .find_journal_by_id(constants::CORE_JOURNAL_ID)
            .await
            .is_ok()
        {
            return Ok(());
        }

        let err = match cala.create_core_journal(constants::CORE_JOURNAL_ID).await {
            Ok(_) => return Ok(()),
            Err(e) => e,
        };

        cala.find_journal_by_id(constants::CORE_JOURNAL_ID)
            .await
            .map_err(|_| err)?;
        Ok(())
    }

    async fn initialize_global_accounts(cala: &CalaClient) -> Result<(), LedgerError> {
        Self::assert_debit_account_exists(
            cala,
            constants::BANK_OFF_BALANCE_SHEET_ID.into(),
            constants::BANK_OFF_BALANCE_SHEET_NAME,
            constants::BANK_OFF_BALANCE_SHEET_CODE,
            &constants::BANK_OFF_BALANCE_SHEET_ID.to_string(),
        )
        .await?;

        Self::assert_debit_account_exists(
            cala,
            constants::BANK_USDT_CASH_ID.into(),
            constants::BANK_USDT_CASH_NAME,
            constants::BANK_USDT_CASH_CODE,
            &constants::BANK_USDT_CASH_ID.to_string(),
        )
        .await?;

        Ok(())
    }

    async fn assert_account_exists(
        normal_balance_type: LedgerDebitOrCredit,
        cala: &CalaClient,
        account_id: LedgerAccountId,
        name: &str,
        code: &str,
        external_id: &str,
    ) -> Result<LedgerAccountId, LedgerError> {
        if let Ok(Some(id)) = cala
            .find_account_by_external_id(external_id.to_owned())
            .await
        {
            return Ok(id);
        }

        let err = match cala
            .create_account(
                account_id,
                normal_balance_type,
                name.to_owned(),
                code.to_owned(),
                external_id.to_owned(),
            )
            .await
        {
            Ok(id) => return Ok(id),
            Err(e) => e,
        };

        cala.find_account_by_external_id(external_id.to_owned())
            .await
            .map_err(|_| err)?
            .ok_or_else(|| LedgerError::CouldNotAssertAccountExits)
    }

    async fn assert_credit_account_exists(
        cala: &CalaClient,
        account_id: LedgerAccountId,
        name: &str,
        code: &str,
        external_id: &str,
    ) -> Result<LedgerAccountId, LedgerError> {
        Self::assert_account_exists(
            LedgerDebitOrCredit::Credit,
            cala,
            account_id,
            name,
            code,
            external_id,
        )
        .await
    }

    async fn assert_debit_account_exists(
        cala: &CalaClient,
        account_id: LedgerAccountId,
        name: &str,
        code: &str,
        external_id: &str,
    ) -> Result<LedgerAccountId, LedgerError> {
        Self::assert_account_exists(
            LedgerDebitOrCredit::Debit,
            cala,
            account_id,
            name,
            code,
            external_id,
        )
        .await
    }

    async fn initialize_tx_templates(cala: &CalaClient) -> Result<(), LedgerError> {
        Self::assert_pledge_unallocated_collateral_tx_template_exists(
            cala,
            constants::PLEDGE_UNALLOCATED_COLLATERAL_CODE,
        )
        .await?;

        Self::assert_deposit_checking_tx_template_exists(cala, constants::DEPOSIT_CHECKING_CODE)
            .await?;

        Self::assert_approve_loan_tx_template_exists(cala, constants::APPROVE_LOAN_CODE).await?;

        Self::assert_incur_interest_tx_template_exists(cala, constants::INCUR_INTEREST_CODE)
            .await?;

        Self::assert_record_payment_tx_template_exists(cala, constants::RECORD_PAYMENT_CODE)
            .await?;

        Self::assert_initiate_withdrawal_from_checking_tx_template_exists(
            cala,
            constants::INITIATE_WITHDRAWAL_FROM_CHECKING_CODE,
        )
        .await?;

        Self::assert_settle_withdrawal_from_checking_tx_template_exists(
            cala,
            constants::SETTLE_WITHDRAWAL_FROM_CHECKING_CODE,
        )
        .await?;

        Self::assert_complete_loan_tx_template_exists(cala, constants::COMPLETE_LOAN_CODE).await?;

        Ok(())
    }

    async fn assert_pledge_unallocated_collateral_tx_template_exists(
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
            .create_pledge_unallocated_collateral_tx_template(template_id)
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

    async fn assert_deposit_checking_tx_template_exists(
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

    async fn assert_initiate_withdrawal_from_checking_tx_template_exists(
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
            .create_initiate_withdrawal_from_checking_tx_template(template_id)
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

    async fn assert_settle_withdrawal_from_checking_tx_template_exists(
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
            .create_settle_withdrawal_from_checking_tx_template(template_id)
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
