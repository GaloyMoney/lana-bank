pub mod error;
mod templates;

use cala_ledger::{
    account::{error::AccountError, *},
    CalaLedger, Currency, DebitOrCredit, JournalId, TransactionId,
};

use crate::{primitives::UsdCents, DepositAccountBalance};

use error::*;

#[derive(Clone)]
pub struct DepositLedger {
    cala: CalaLedger,
    journal_id: JournalId,
    deposit_omnibus_account_id: AccountId,
    usd: Currency,
}

impl DepositLedger {
    pub async fn init(
        cala: &CalaLedger,
        journal_id: JournalId,
        omnibus_account_code: String,
    ) -> Result<Self, DepositLedgerError> {
        let deposit_omnibus_account_id =
            Self::create_deposit_omnibus_account(cala, omnibus_account_code.clone()).await?;

        templates::RecordDeposit::init(cala).await?;
        templates::InitiateWithdraw::init(cala).await?;
        templates::CancelWithdraw::init(cala).await?;
        templates::ConfirmWithdraw::init(cala).await?;

        Ok(Self {
            cala: cala.clone(),
            journal_id,
            deposit_omnibus_account_id,
            usd: "USD".parse().expect("Could not parse 'USD'"),
        })
    }

    pub async fn create_account_for_deposit_account(
        &self,
        op: es_entity::DbOp<'_>,
        id: impl Into<AccountId>,
        code: String,
    ) -> Result<(), DepositLedgerError> {
        let mut op = self.cala.ledger_operation_from_db_op(op);

        let new_account = NewAccount::builder()
            .id(id)
            .name("Deposit Account")
            .code(code)
            .build()
            .expect("Could not build new account");

        self.cala
            .accounts()
            .create_in_op(&mut op, new_account)
            .await?;

        op.commit().await?;

        Ok(())
    }

    pub async fn record_deposit(
        &self,
        op: es_entity::DbOp<'_>,
        tx_id: impl Into<TransactionId>,
        amount: UsdCents,
        credit_account_id: impl Into<AccountId>,
    ) -> Result<(), DepositLedgerError> {
        let tx_id = tx_id.into();
        let mut op = self.cala.ledger_operation_from_db_op(op);

        let params = templates::RecordDepositParams {
            journal_id: self.journal_id,
            currency: self.usd,
            amount: amount.to_usd(),
            deposit_omnibus_account_id: self.deposit_omnibus_account_id,
            credit_account_id: credit_account_id.into(),
        };
        self.cala
            .post_transaction_in_op(&mut op, tx_id, templates::RECORD_DEPOSIT_CODE, params)
            .await?;

        op.commit().await?;
        Ok(())
    }

    pub async fn initiate_withdrawal(
        &self,
        op: es_entity::DbOp<'_>,
        tx_id: impl Into<TransactionId>,
        amount: UsdCents,
        credit_account_id: impl Into<AccountId>,
    ) -> Result<(), DepositLedgerError> {
        let tx_id = tx_id.into();
        let mut op = self.cala.ledger_operation_from_db_op(op);

        let params = templates::InitiateWithdrawParams {
            journal_id: self.journal_id,
            deposit_omnibus_account_id: self.deposit_omnibus_account_id,
            credit_account_id: credit_account_id.into(),
            amount: amount.to_usd(),
            currency: self.usd,
        };

        self.cala
            .post_transaction_in_op(&mut op, tx_id, templates::INITIATE_WITHDRAW_CODE, params)
            .await?;

        op.commit().await?;
        Ok(())
    }

    pub async fn confirm_withdrawal(
        &self,
        op: es_entity::DbOp<'_>,
        tx_id: impl Into<TransactionId>,
        correlation_id: String,
        amount: UsdCents,
        credit_account_id: impl Into<AccountId>,
        external_id: String,
    ) -> Result<(), DepositLedgerError> {
        let tx_id = tx_id.into();
        let mut op = self.cala.ledger_operation_from_db_op(op);

        let params = templates::ConfirmWithdrawParams {
            journal_id: self.journal_id,
            currency: self.usd,
            amount: amount.to_usd(),
            deposit_omnibus_account_id: self.deposit_omnibus_account_id,
            credit_account_id: credit_account_id.into(),
            correlation_id,
            external_id,
        };

        self.cala
            .post_transaction_in_op(&mut op, tx_id, templates::CONFIRM_WITHDRAW_CODE, params)
            .await?;
        op.commit().await?;
        Ok(())
    }

    pub async fn cancel_withdrawal(
        &self,
        op: es_entity::DbOp<'_>,
        tx_id: impl Into<TransactionId>,
        amount: UsdCents,
        credit_account_id: impl Into<AccountId>,
    ) -> Result<(), DepositLedgerError> {
        let tx_id = tx_id.into();
        let mut op = self.cala.ledger_operation_from_db_op(op);

        let params = templates::CancelWithdrawParams {
            journal_id: self.journal_id,
            currency: self.usd,
            amount: amount.to_usd(),
            credit_account_id: credit_account_id.into(),
            deposit_omnibus_account_id: self.deposit_omnibus_account_id,
        };

        self.cala
            .post_transaction_in_op(&mut op, tx_id, templates::CANCEL_WITHDRAW_CODE, params)
            .await?;
        op.commit().await?;
        Ok(())
    }

    async fn create_deposit_omnibus_account(
        cala: &CalaLedger,
        code: String,
    ) -> Result<AccountId, DepositLedgerError> {
        let new_account = NewAccount::builder()
            .code(&code)
            .id(AccountId::new())
            .name("Deposit Omnibus Account")
            .description("Omnibus Account for Deposit module")
            .normal_balance_type(DebitOrCredit::Debit)
            .build()
            .expect("Couldn't create onchain incoming account");
        match cala.accounts().create(new_account).await {
            Err(AccountError::CodeAlreadyExists) => {
                let account = cala.accounts().find_by_code(code).await?;
                Ok(account.id)
            }
            Err(e) => Err(e.into()),
            Ok(account) => Ok(account.id),
        }
    }

    pub async fn balance(
        &self,
        account_id: impl Into<AccountId>,
    ) -> Result<DepositAccountBalance, DepositLedgerError> {
        let balances = self
            .cala
            .balances()
            .find(self.journal_id, account_id.into(), self.usd)
            .await?;

        Ok(DepositAccountBalance {
            settled: UsdCents::try_from_usd(balances.settled())?,
            pending: UsdCents::try_from_usd(balances.pending())?,
        })
    }
}
