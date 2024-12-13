mod credit_facility_accounts;
pub mod error;
mod templates;

use cala_ledger::{
    account::{error::AccountError, NewAccount},
    AccountId, CalaLedger, Currency, DebitOrCredit, JournalId, TransactionId,
};

use crate::primitives::{CollateralAction, CreditFacilityId, Satoshis, UsdCents};

pub use credit_facility_accounts::*;
use error::*;

pub(super) const BANK_COLLATERAL_ACCOUNT_CODE: &str = "BANK.COLLATERAL.OMNIBUS";

#[derive(Debug, Clone)]
pub struct CreditFacilityCollateralUpdate {
    pub tx_id: TransactionId,
    pub abs_diff: Satoshis,
    pub action: CollateralAction,
    pub credit_facility_account_ids: CreditFacilityAccountIds,
}

#[derive(Clone)]
pub struct CreditLedger {
    cala: CalaLedger,
    journal_id: JournalId,
    bank_collateral_account_id: AccountId,
    usd: Currency,
    btc: Currency,
}

impl CreditLedger {
    pub async fn init(cala: &CalaLedger, journal_id: JournalId) -> Result<Self, CreditLedgerError> {
        let bank_collateral_account_id =
            Self::create_bank_collateral_account(cala, BANK_COLLATERAL_ACCOUNT_CODE.to_string())
                .await?;
        templates::AddCollateral::init(cala).await?;
        templates::RemoveCollateral::init(cala).await?;

        Ok(Self {
            cala: cala.clone(),
            journal_id,
            bank_collateral_account_id,
            usd: "USD".parse().expect("Could not parse 'USD'"),
            btc: "BTC".parse().expect("Could not parse 'BTC'"),
        })
    }

    pub async fn create_accounts_for_credit_facility(
        &self,
        op: es_entity::DbOp<'_>,
        credit_facility_id: CreditFacilityId,
        CreditFacilityAccountIds {
            facility_account_id,
            disbursed_receivable_account_id,
            collateral_account_id,
            interest_receivable_account_id,
            interest_account_id,
            fee_income_account_id,
        }: CreditFacilityAccountIds,
    ) -> Result<(), CreditLedgerError> {
        let mut op = self.cala.ledger_operation_from_db_op(op);
        let new_accounts = vec![
            NewAccount::builder()
                .id(collateral_account_id)
                .name("Credit Facility Collateral Account")
                .code(format!("CREDIT_FACILITY.COLLATERAL.{}", credit_facility_id))
                .build()
                .expect("new account"),
            NewAccount::builder()
                .id(facility_account_id)
                .name("Off-Balance-Sheet Facility Account for Credit Facility")
                .code(format!(
                    "CREDIT_FACILITY.OBS_FACILITY.{}",
                    credit_facility_id
                ))
                .build()
                .expect("new account"),
            NewAccount::builder()
                .id(disbursed_receivable_account_id)
                .name("Disbursed Receivable Account for Credit Facility")
                .code(format!(
                    "CREDIT_FACILITY.DISBURSED_RECEIVABLE.{}",
                    credit_facility_id
                ))
                .build()
                .expect("new account"),
            NewAccount::builder()
                .id(interest_receivable_account_id)
                .name("Interest Receivable Account for Credit Facility")
                .code(format!(
                    "CREDIT_FACILITY.INTEREST_RECEIVABLE.{}",
                    credit_facility_id
                ))
                .build()
                .expect("new account"),
            NewAccount::builder()
                .id(interest_account_id)
                .name("Interest Income for Credit Facility")
                .code(format!(
                    "CREDIT_FACILITY.INTEREST_INCOME.{}",
                    credit_facility_id
                ))
                .build()
                .expect("new account"),
            NewAccount::builder()
                .id(fee_income_account_id)
                .name("Fee Income for Credit Facility")
                .code(format!("CREDIT_FACILITY.FEE_INCOME.{}", credit_facility_id))
                .build()
                .expect("new account"),
        ];

        self.cala
            .accounts()
            .create_all_in_op(&mut op, new_accounts)
            .await?;

        op.commit().await?;

        Ok(())
    }

    pub async fn get_credit_facility_balance(
        &self,
        CreditFacilityAccountIds {
            facility_account_id,
            disbursed_receivable_account_id,
            collateral_account_id,
            interest_receivable_account_id,
            ..
        }: CreditFacilityAccountIds,
    ) -> Result<CreditFacilityLedgerBalance, CreditLedgerError> {
        let facility_id = (self.journal_id, facility_account_id, self.usd);
        let collateral_id = (self.journal_id, collateral_account_id, self.btc);
        let disbursed_receivable_id = (self.journal_id, disbursed_receivable_account_id, self.btc);
        let interest_receivable_id = (self.journal_id, interest_receivable_account_id, self.btc);
        let balances = self
            .cala
            .balances()
            .find_all(&[
                facility_id,
                collateral_id,
                disbursed_receivable_id,
                interest_receivable_id,
            ])
            .await?;
        let facility = if let Some(b) = balances.get(&facility_id) {
            UsdCents::try_from_usd(b.settled())?
        } else {
            UsdCents::ZERO
        };
        let disbursed = if let Some(b) = balances.get(&disbursed_receivable_id) {
            UsdCents::try_from_usd(b.details.settled.dr_balance)?
        } else {
            UsdCents::ZERO
        };
        let disbursed_receivable = if let Some(b) = balances.get(&disbursed_receivable_id) {
            UsdCents::try_from_usd(b.settled())?
        } else {
            UsdCents::ZERO
        };
        let interest = if let Some(b) = balances.get(&interest_receivable_id) {
            UsdCents::try_from_usd(b.details.settled.dr_balance)?
        } else {
            UsdCents::ZERO
        };
        let interest_receivable = if let Some(b) = balances.get(&interest_receivable_id) {
            UsdCents::try_from_usd(b.settled())?
        } else {
            UsdCents::ZERO
        };
        let collateral = if let Some(b) = balances.get(&collateral_id) {
            Satoshis::try_from_btc(b.settled())?
        } else {
            Satoshis::ZERO
        };
        Ok(CreditFacilityLedgerBalance {
            facility,
            collateral,
            disbursed,
            disbursed_receivable,
            interest,
            interest_receivable,
        })
    }

    pub async fn update_credit_facility_collateral(
        &self,
        op: es_entity::DbOp<'_>,
        CreditFacilityCollateralUpdate {
            tx_id,
            credit_facility_account_ids,
            abs_diff,
            action,
        }: CreditFacilityCollateralUpdate,
    ) -> Result<(), CreditLedgerError> {
        let mut op = self.cala.ledger_operation_from_db_op(op);
        match action {
            CollateralAction::Add => {
                self.cala
                    .post_transaction_in_op(
                        &mut op,
                        tx_id,
                        templates::ADD_COLLATERAL_CODE,
                        templates::AddCollateralParams {
                            journal_id: self.journal_id,
                            currency: self.btc,
                            amount: abs_diff.to_btc(),
                            collateral_account_id: credit_facility_account_ids
                                .collateral_account_id,
                            bank_collateral_account_id: self.bank_collateral_account_id,
                        },
                    )
                    .await
            }
            CollateralAction::Remove => {
                self.cala
                    .post_transaction_in_op(
                        &mut op,
                        tx_id,
                        templates::REMOVE_COLLATERAL_CODE,
                        templates::RemoveCollateralParams {
                            journal_id: self.journal_id,
                            currency: self.btc,
                            amount: abs_diff.to_btc(),
                            collateral_account_id: credit_facility_account_ids
                                .collateral_account_id,
                            bank_collateral_account_id: self.bank_collateral_account_id,
                        },
                    )
                    .await
            }
        }?;
        op.commit().await?;
        Ok(())
    }

    async fn create_bank_collateral_account(
        cala: &CalaLedger,
        code: String,
    ) -> Result<AccountId, CreditLedgerError> {
        let new_account = NewAccount::builder()
            .code(&code)
            .id(AccountId::new())
            .name("Bank collateral account")
            .description("Bank collateral account")
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
}
