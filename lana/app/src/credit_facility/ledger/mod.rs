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
pub(super) const CREDIT_OMNIBUS_ACCOUNT_CODE: &str = "CREDIT.OMNIBUS";

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
    credit_omnibus_account: AccountId,
    bank_collateral_account_id: AccountId,
    usd: Currency,
    btc: Currency,
}

impl CreditLedger {
    pub async fn init(cala: &CalaLedger, journal_id: JournalId) -> Result<Self, CreditLedgerError> {
        let bank_collateral_account_id =
            Self::create_bank_collateral_account(cala, BANK_COLLATERAL_ACCOUNT_CODE.to_string())
                .await?;

        let credit_omnibus_account =
            Self::create_credit_omnibus_account(cala, CREDIT_OMNIBUS_ACCOUNT_CODE.to_string())
                .await?;

        templates::AddCollateral::init(cala).await?;
        templates::ApproveCreditFacility::init(cala).await?;
        templates::RemoveCollateral::init(cala).await?;
        templates::RecordPayment::init(cala).await?;
        templates::CreditFacilityIncurInterest::init(cala).await?;
        templates::CreditFacilityAccrueInterest::init(cala).await?;
        templates::CreditFacilityDisbursal::init(cala).await?;

        Ok(Self {
            cala: cala.clone(),
            journal_id,
            bank_collateral_account_id,
            credit_omnibus_account,
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

    pub async fn record_credit_facility_repayment(
        &self,
        op: es_entity::DbOp<'_>,
        CreditFacilityRepayment {
            tx_id,
            tx_ref,
            credit_facility_account_ids,
            debit_account_id,
            amounts,
        }: CreditFacilityRepayment,
    ) -> Result<(), CreditLedgerError> {
        let mut op = self.cala.ledger_operation_from_db_op(op);

        let params = templates::RecordPaymentParams {
            journal_id: self.journal_id,
            currency: self.usd,
            interest_amount: amounts.interest.to_usd(),
            principal_amount: amounts.disbursal.to_usd(),
            debit_account_id,
            principal_receivable_account_id: credit_facility_account_ids
                .disbursed_receivable_account_id,
            interest_receivable_account_id: credit_facility_account_ids
                .interest_receivable_account_id,
            tx_ref,
        };
        self.cala
            .post_transaction_in_op(&mut op, tx_id, templates::RECORD_PAYMENT_CODE, params)
            .await?;

        op.commit().await?;
        Ok(())
    }

    pub async fn complete_credit_facility(
        &self,
        op: es_entity::DbOp<'_>,
        CreditFacilityCompletion {
            tx_id,
            collateral,
            credit_facility_account_ids,
        }: CreditFacilityCompletion,
    ) -> Result<(), CreditLedgerError> {
        let mut op = self.cala.ledger_operation_from_db_op(op);
        self.cala
            .post_transaction_in_op(
                &mut op,
                tx_id,
                templates::REMOVE_COLLATERAL_CODE,
                templates::RemoveCollateralParams {
                    journal_id: self.journal_id,
                    currency: self.btc,
                    amount: collateral.to_btc(),
                    collateral_account_id: credit_facility_account_ids.collateral_account_id,
                    bank_collateral_account_id: self.bank_collateral_account_id,
                },
            )
            .await?;
        op.commit().await?;
        Ok(())
    }

    pub async fn activate_credit_facility(
        &self,
        op: es_entity::DbOp<'_>,
        CreditFacilityActivation {
            tx_id,
            tx_ref,
            credit_facility_account_ids,
            debit_account_id,
            facility_amount,
            structuring_fee_amount,
        }: CreditFacilityActivation,
    ) -> Result<(), CreditLedgerError> {
        let mut op = self.cala.ledger_operation_from_db_op(op);
        self.cala
            .post_transaction_in_op(
                &mut op,
                tx_id,
                templates::APPROVE_CREDIT_FACILITY_CODE,
                templates::ApproveCreditFacilityParams {
                    journal_id: self.journal_id,
                    credit_omnibus_account: self.credit_omnibus_account,
                    credit_facility_account: credit_facility_account_ids.facility_account_id,
                    facility_disbursed_receivable_account: credit_facility_account_ids
                        .disbursed_receivable_account_id,
                    facility_fee_income_account: credit_facility_account_ids.fee_income_account_id,
                    checking_account: debit_account_id,
                    facility_amount: facility_amount.to_usd(),
                    structuring_fee_amount: structuring_fee_amount.to_usd(),
                    currency: self.usd,
                    external_id: tx_ref,
                },
            )
            .await?;
        op.commit().await?;
        Ok(())
    }

    pub async fn record_interest_incurrence(
        &self,
        op: es_entity::DbOp<'_>,
        CreditFacilityInterestIncurrence {
            tx_id,
            tx_ref,
            interest,
            period,
            credit_facility_account_ids,
        }: CreditFacilityInterestIncurrence,
    ) -> Result<(), CreditLedgerError> {
        let mut op = self.cala.ledger_operation_from_db_op(op);
        self.cala
            .post_transaction_in_op(
                &mut op,
                tx_id,
                templates::CREDIT_FACILITY_INCUR_INTEREST_CODE,
                templates::CreditFacilityIncurInterestParams {
                    journal_id: self.journal_id,

                    credit_facility_interest_receivable_account: credit_facility_account_ids
                        .interest_receivable_account_id,
                    credit_facility_interest_income_account: credit_facility_account_ids
                        .interest_account_id,
                    interest_amount: interest.to_usd(),
                    external_id: tx_ref,
                    effective: period.end.date_naive(),
                },
            )
            .await?;
        op.commit().await?;
        Ok(())
    }

    pub async fn record_interest_accrual(
        &self,
        op: es_entity::DbOp<'_>,
        CreditFacilityInterestAccrual {
            tx_id,
            tx_ref,
            interest,
            credit_facility_account_ids,
            accrued_at,
        }: CreditFacilityInterestAccrual,
    ) -> Result<(), CreditLedgerError> {
        let mut op = self.cala.ledger_operation_from_db_op(op);
        self.cala
            .post_transaction_in_op(
                &mut op,
                tx_id,
                templates::CREDIT_FACILITY_ACCRUE_INTEREST_CODE,
                templates::CreditFacilityAccrueInterestParams {
                    journal_id: self.journal_id,

                    credit_facility_interest_receivable_account: credit_facility_account_ids
                        .interest_receivable_account_id,
                    credit_facility_interest_income_account: credit_facility_account_ids
                        .interest_account_id,
                    interest_amount: interest.to_usd(),
                    external_id: tx_ref,
                    effective: accrued_at.date_naive(),
                },
            )
            .await?;
        op.commit().await?;
        Ok(())
    }

    pub async fn record_disbursal(
        &self,
        op: es_entity::DbOp<'_>,
        DisbursalData {
            tx_id,
            tx_ref,
            amount,
            credit_facility_account_ids,
            debit_account_id,
        }: DisbursalData,
    ) -> Result<(), CreditLedgerError> {
        let mut op = self.cala.ledger_operation_from_db_op(op);
        self.cala
            .post_transaction_in_op(
                &mut op,
                tx_id,
                templates::CREDIT_FACILITY_DISBURSAL_CODE,
                templates::CreditFacilityDisbursalParams {
                    journal_id: self.journal_id,
                    credit_omnibus_account: self.credit_omnibus_account,
                    credit_facility_account: credit_facility_account_ids.facility_account_id,
                    facility_disbursed_receivable_account: credit_facility_account_ids
                        .disbursed_receivable_account_id,
                    checking_account: debit_account_id,
                    disbursed_amount: amount.to_usd(),
                    external_id: tx_ref,
                },
            )
            .await?;
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

    async fn create_credit_omnibus_account(
        cala: &CalaLedger,
        code: String,
    ) -> Result<AccountId, CreditLedgerError> {
        let new_account = NewAccount::builder()
            .code(&code)
            .id(AccountId::new())
            .name("Credit Omnibus Account")
            .description("Omnibus Account for Credit module")
            .normal_balance_type(DebitOrCredit::Debit)
            .build()
            .expect("Couldn't create credit omnibus account");
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
