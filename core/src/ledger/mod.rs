mod account;
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

use account::LedgerAccount;
use bitfinex::BfxIntegration;
use tracing::instrument;

use crate::primitives::{
    BfxAddressType, BfxIntegrationId, BfxWithdrawalMethod, FixedTermLoanId, LedgerAccountId,
    LedgerAccountSetId, LedgerDebitOrCredit, LedgerTxId, LedgerTxTemplateId, Satoshis, UsdCents,
    WithdrawId,
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
        Self::initialize_bfx_integrations(&cala, &config.bfx_key, &config.bfx_secret).await?;
        Self::initialize_global_accounts(&cala).await?;
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
        bitfinex_username: &str,
    ) -> Result<(UserLedgerAccountIds, UserLedgerAccountAddresses), LedgerError> {
        let account_ids = UserLedgerAccountIds::new();
        Self::assert_credit_account_exists(
            &self.cala,
            account_ids.unallocated_collateral_id,
            &format!("USERS.UNALLOCATED_COLLATERAL.{}", bitfinex_username),
            &format!("USERS.UNALLOCATED_COLLATERAL.{}", bitfinex_username),
            &format!("usr:bfx-{}:unallocated_collateral", bitfinex_username),
        )
        .await?;

        let unallocated_collateral_address =
            Self::assert_off_balance_sheet_address_backed_debit_account_exists(
                &self.cala,
                account_ids.bank_unallocated_collateral_id, // TODO: revisit if this should be on user entity
                &format!("BANK.USER_UNALLOCATED_COLLATERAL.{}", bitfinex_username),
                &format!("BANK.USER_UNALLOCATED_COLLATERAL.{}", bitfinex_username),
                account_ids.unallocated_collateral_id,
            )
            .await?;

        // FIXME: assert is in off-balance-sheet integration omnibus
        // Self::assert_account_in_account_set(
        //     cala,
        //     constants::BANK_OFF_BALANCE_SHEET_ACCOUNT_SET_ID.into(),
        //     account_ids.bank_unallocated_collateral_id,
        // )
        // .await?;

        Self::assert_credit_account_exists(
            &self.cala,
            account_ids.checking_id,
            &format!("USERS.CHECKING.{}", bitfinex_username),
            &format!("USERS.CHECKING.{}", bitfinex_username),
            &format!("usr:bfx-{}:checking", bitfinex_username),
        )
        .await?;

        let checking_address = Self::assert_usdt_cash_address_backed_debit_account_exists(
            &self.cala,
            account_ids.bank_checking_id, // TODO: revisit if this should be on user entity
            &format!("BANK.USER_CHECKING.{}", bitfinex_username),
            &format!("BANK.USER_CHECKING.{}", bitfinex_username),
            account_ids.checking_id,
        )
        .await?;

        // FIXME: assert is in usdt-cash integration omnibus
        // Self::assert_account_in_account_set(
        //     cala,
        //     constants::BANK_OFF_BALANCE_SHEET_ACCOUNT_SET_ID.into(),
        //     account_ids.bank_checking_id,
        // )
        // .await?;

        let account_addresses = UserLedgerAccountAddresses {
            checking_address,
            unallocated_collateral_address,
        };

        Ok((account_ids, account_addresses))
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
                constants::BITFINEX_USDT_CASH_INTEGRATION_ID.into(),
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

    pub async fn btc_equity_address_for_address_backed_account_by_id(
        &self,
    ) -> Result<Option<String>, LedgerError> {
        self.cala
            .find_address_backed_account_by_id(constants::BANK_BTC_RESERVE_FROM_SHAREHOLDER_ID)
            .await
            .map_err(|err| err.into())
    }

    async fn initialize_global_accounts(cala: &CalaClient) -> Result<(), LedgerError> {
        Self::assert_shareholder_btc_address_backed_debit_account_exists(
            cala,
            constants::BANK_BTC_RESERVE_FROM_SHAREHOLDER_ID.into(),
            constants::BANK_BTC_RESERVE_FROM_SHAREHOLDER_NAME,
            constants::BANK_BTC_RESERVE_FROM_SHAREHOLDER_CODE,
            constants::BANK_SHAREHOLDER_EQUITY_ID.into(),
        )
        .await?;

        Ok(())
    }

    async fn _assert_account_set_exists(
        normal_balance_type: LedgerDebitOrCredit,
        cala: &CalaClient,
        account_set_id: LedgerAccountSetId,
        name: &str,
    ) -> Result<LedgerAccountSetId, LedgerError> {
        if let Ok(Some(id)) = cala.find_account_set_by_id(account_set_id.to_owned()).await {
            return Ok(id);
        }

        let err = match cala
            .create_account_set(account_set_id, name.to_owned(), normal_balance_type)
            .await
        {
            Ok(id) => return Ok(id),
            Err(e) => e,
        };

        cala.find_account_set_by_id(account_set_id.to_owned())
            .await
            .map_err(|_| err)?
            .ok_or_else(|| LedgerError::CouldNotAssertAccountSetExists)
    }

    async fn _assert_debit_account_set_exists(
        cala: &CalaClient,
        account_set_id: LedgerAccountSetId,
        name: &str,
    ) -> Result<LedgerAccountSetId, LedgerError> {
        Self::_assert_account_set_exists(LedgerDebitOrCredit::Debit, cala, account_set_id, name)
            .await
    }

    async fn _assert_account_in_account_set(
        cala: &CalaClient,
        account_set_id: LedgerAccountSetId,
        account_id: LedgerAccountId,
    ) -> Result<LedgerAccountSetId, LedgerError> {
        if let Ok(Some(ledger_account)) = cala.find_account_by_id::<LedgerAccount>(account_id).await
        {
            if ledger_account.account_set_ids.contains(&account_set_id) {
                return Ok(account_set_id);
            }
        }

        let err = match cala
            .add_account_to_account_set(account_set_id, account_id)
            .await
        {
            Ok(id) => return Ok(id),
            Err(e) => e,
        };

        match cala
            .find_account_by_id::<LedgerAccount>(account_id)
            .await
            .map_err(|_| err)
        {
            Ok(Some(_)) => Ok(account_set_id),
            Ok(None) => Err(LedgerError::CouldNotAssertAccountIsMemberOfAccountSet),
            Err(e) => Err(e)?,
        }
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
            .ok_or_else(|| LedgerError::CouldNotAssertAccountExists)
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

    async fn assert_address_backed_debit_account_exists(
        integration_id: BfxIntegrationId,
        address_type: BfxAddressType,
        cala: &CalaClient,
        account_id: LedgerAccountId,
        name: &str,
        code: &str,
        credit_account_id: LedgerAccountId,
    ) -> Result<String, LedgerError> {
        if let Ok(Some(address)) = cala.find_address_backed_account_by_id(account_id).await {
            return Ok(address);
        }

        let err = match cala
            .create_bfx_address_backed_account(
                integration_id,
                address_type,
                account_id,
                name.to_owned(),
                code.to_owned(),
                credit_account_id,
            )
            .await
        {
            Ok(address) => return Ok(address),
            Err(e) => e,
        };

        match cala.find_address_backed_account_by_id(account_id).await {
            Ok(Some(address)) => Ok(address),
            Ok(None) => Err(LedgerError::CouldNotAssertAccountExists),
            Err(_) => Err(err)?,
        }
    }

    async fn assert_shareholder_btc_address_backed_debit_account_exists(
        cala: &CalaClient,
        account_id: LedgerAccountId,
        name: &str,
        code: &str,
        credit_account_id: LedgerAccountId,
    ) -> Result<String, LedgerError> {
        Self::assert_address_backed_debit_account_exists(
            constants::BITFINEX_SHAREHOLDER_INTEGRATION_ID.into(),
            BfxAddressType::Bitcoin,
            cala,
            account_id,
            name,
            code,
            credit_account_id,
        )
        .await
    }

    async fn assert_off_balance_sheet_address_backed_debit_account_exists(
        cala: &CalaClient,
        account_id: LedgerAccountId,
        name: &str,
        code: &str,
        credit_account_id: LedgerAccountId,
    ) -> Result<String, LedgerError> {
        Self::assert_address_backed_debit_account_exists(
            constants::BITFINEX_OFF_BALANCE_SHEET_INTEGRATION_ID.into(),
            BfxAddressType::Bitcoin,
            cala,
            account_id,
            name,
            code,
            credit_account_id,
        )
        .await
    }

    async fn assert_usdt_cash_address_backed_debit_account_exists(
        cala: &CalaClient,
        account_id: LedgerAccountId,
        name: &str,
        code: &str,
        credit_account_id: LedgerAccountId,
    ) -> Result<String, LedgerError> {
        Self::assert_address_backed_debit_account_exists(
            constants::BITFINEX_USDT_CASH_INTEGRATION_ID.into(),
            BfxAddressType::Tron,
            cala,
            account_id,
            name,
            code,
            credit_account_id,
        )
        .await
    }

    async fn initialize_tx_templates(cala: &CalaClient) -> Result<(), LedgerError> {
        Self::assert_approve_loan_tx_template_exists(cala, constants::APPROVE_LOAN_CODE).await?;

        Self::assert_incur_interest_tx_template_exists(cala, constants::INCUR_INTEREST_CODE)
            .await?;

        Self::assert_record_payment_tx_template_exists(cala, constants::RECORD_PAYMENT_CODE)
            .await?;

        Self::assert_complete_loan_tx_template_exists(cala, constants::COMPLETE_LOAN_CODE).await?;

        Ok(())
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

    async fn assert_bfx_integration_exists(
        cala: &CalaClient,
        bfx_integration_id: BfxIntegrationId,
        name: &str,
        key: &str,
        secret: &str,
    ) -> Result<BfxIntegrationId, LedgerError> {
        if let Ok(Some(bfx_integration)) = cala
            .find_bfx_integration_by_id::<BfxIntegration>(bfx_integration_id.to_owned())
            .await
        {
            return Ok(bfx_integration.id);
        }

        let err = match cala
            .create_bfx_integration(
                bfx_integration_id,
                name.to_string(),
                key.to_string(),
                secret.to_string(),
            )
            .await
        {
            Ok(bfx_integration) => {
                return Ok(bfx_integration.id);
            }
            Err(e) => e,
        };

        cala.find_bfx_integration_by_id::<BfxIntegration>(bfx_integration_id.to_owned())
            .await
            .map_err(|_| err)?
            .ok_or_else(|| LedgerError::CouldNotAssertBfxIntegrationExists)
            .map(|bfx_integration| bfx_integration.id)
    }

    async fn initialize_bfx_integrations(
        cala: &CalaClient,
        key: &str,
        secret: &str,
    ) -> Result<(), LedgerError> {
        Self::assert_bfx_integration_exists(
            cala,
            constants::BITFINEX_OFF_BALANCE_SHEET_INTEGRATION_ID.into(),
            constants::BITFINEX_OFF_BALANCE_SHEET_INTEGRATION_NAME,
            key,
            secret,
        )
        .await?;

        Self::assert_bfx_integration_exists(
            cala,
            constants::BITFINEX_USDT_CASH_INTEGRATION_ID.into(),
            constants::BITFINEX_USDT_CASH_INTEGRATION_NAME,
            key,
            secret,
        )
        .await?;

        Self::assert_bfx_integration_exists(
            cala,
            constants::BITFINEX_SHAREHOLDER_INTEGRATION_ID.into(),
            constants::BITFINEX_SHAREHOLDER_INTEGRATION_NAME,
            key,
            secret,
        )
        .await?;

        Ok(())
    }
}
