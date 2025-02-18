mod credit_facility_accounts;
pub mod error;
mod templates;
mod velocity;

use cala_ledger::{
    velocity::{NewVelocityControl, VelocityControlId},
    AccountId, CalaLedger, Currency, JournalId, TransactionId,
};
use chart_of_accounts::TransactionAccountFactory;

use crate::primitives::{CollateralAction, CreditFacilityId, Satoshis, UsdCents};

pub use credit_facility_accounts::*;
use error::*;

pub(super) const CREDIT_FACILITY_VELOCITY_CONTROL_ID: uuid::Uuid =
    uuid::uuid!("00000000-0000-0000-0000-000000000002");

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
    credit_omnibus_account_id: AccountId,
    bank_collateral_account_id: AccountId,
    credit_facility_control_id: VelocityControlId,
    account_factories: CreditFacilityAccountFactories,
    usd: Currency,
    btc: Currency,
}

impl CreditLedger {
    pub async fn init(
        cala: &CalaLedger,
        journal_id: JournalId,
        account_factories: CreditFacilityAccountFactories,
    ) -> Result<Self, CreditLedgerError> {
        let bank_collateral_account_id = Self::create_bank_collateral_account(
            cala,
            account_factories.collateral_omnibus.clone(),
        )
        .await?;

        let credit_omnibus_account_id =
            Self::create_credit_omnibus_account(cala, account_factories.facility_omnibus.clone())
                .await?;

        templates::AddCollateral::init(cala).await?;
        templates::ApproveCreditFacility::init(cala).await?;
        templates::RemoveCollateral::init(cala).await?;
        templates::RecordPayment::init(cala).await?;
        templates::CreditFacilityIncurInterest::init(cala).await?;
        templates::CreditFacilityAccrueInterest::init(cala).await?;
        templates::InitiateDisbursal::init(cala).await?;
        templates::CancelDisbursal::init(cala).await?;
        templates::SettleDisbursal::init(cala).await?;

        let disbursal_limit_id = velocity::DisbursalLimit::init(cala).await?;

        let credit_facility_control_id = Self::create_credit_facility_control(cala).await?;

        match cala
            .velocities()
            .add_limit_to_control(credit_facility_control_id, disbursal_limit_id)
            .await
        {
            Ok(_)
            | Err(cala_ledger::velocity::error::VelocityError::LimitAlreadyAddedToControl) => {}
            Err(e) => return Err(e.into()),
        }

        Ok(Self {
            cala: cala.clone(),
            journal_id,
            bank_collateral_account_id,
            credit_omnibus_account_id,
            credit_facility_control_id,
            account_factories,
            usd: "USD".parse().expect("Could not parse 'USD'"),
            btc: "BTC".parse().expect("Could not parse 'BTC'"),
        })
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
        let disbursed_receivable_id = (self.journal_id, disbursed_receivable_account_id, self.usd);
        let interest_receivable_id = (self.journal_id, interest_receivable_account_id, self.usd);
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
        tx_id: TransactionId,
        tx_ref: String,
        amounts: CreditFacilityPaymentAmounts,
        credit_facility_account_ids: CreditFacilityAccountIds,
        debit_account_id: AccountId,
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
                    credit_omnibus_account: self.credit_omnibus_account_id,
                    credit_facility_account: credit_facility_account_ids.facility_account_id,
                    facility_disbursed_receivable_account: credit_facility_account_ids
                        .disbursed_receivable_account_id,
                    facility_fee_income_account: credit_facility_account_ids.fee_income_account_id,
                    debit_account_id,
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

    pub async fn initiate_disbursal(
        &self,
        op: es_entity::DbOp<'_>,
        tx_id: impl Into<TransactionId>,
        amount: UsdCents,
        credit_facility_account_ids: CreditFacilityAccountIds,
    ) -> Result<(), CreditLedgerError> {
        let mut op = self.cala.ledger_operation_from_db_op(op);
        self.cala
            .post_transaction_in_op(
                &mut op,
                tx_id.into(),
                templates::INITIATE_DISBURSAL_CODE,
                templates::InitiateDisbursalParams {
                    journal_id: self.journal_id,
                    credit_omnibus_account: self.credit_omnibus_account_id,
                    credit_facility_account: credit_facility_account_ids.facility_account_id,
                    disbursed_amount: amount.to_usd(),
                },
            )
            .await?;
        op.commit().await?;
        Ok(())
    }

    pub async fn conclude_disbursal(
        &self,
        op: es_entity::DbOp<'_>,
        DisbursalData {
            tx_id,
            tx_ref,
            amount,
            credit_facility_account_ids,
            debit_account_id,
            cancelled: canceled,
        }: DisbursalData,
    ) -> Result<(), CreditLedgerError> {
        let mut op = self.cala.ledger_operation_from_db_op(op);
        if canceled {
            self.cala
                .post_transaction_in_op(
                    &mut op,
                    tx_id,
                    templates::CANCEL_DISBURSAL_CODE,
                    templates::CancelDisbursalParams {
                        journal_id: self.journal_id,
                        credit_omnibus_account: self.credit_omnibus_account_id,
                        credit_facility_account: credit_facility_account_ids.facility_account_id,
                        disbursed_amount: amount.to_usd(),
                    },
                )
                .await?;
        } else {
            self.cala
                .post_transaction_in_op(
                    &mut op,
                    tx_id,
                    templates::SETTLE_DISBURSAL_CODE,
                    templates::SettleDisbursalParams {
                        journal_id: self.journal_id,
                        credit_omnibus_account: self.credit_omnibus_account_id,
                        credit_facility_account: credit_facility_account_ids.facility_account_id,
                        facility_disbursed_receivable_account: credit_facility_account_ids
                            .disbursed_receivable_account_id,
                        debit_account_id,
                        disbursed_amount: amount.to_usd(),
                        external_id: tx_ref,
                    },
                )
                .await?;
        }
        op.commit().await?;
        Ok(())
    }

    async fn create_bank_collateral_account(
        cala: &CalaLedger,
        account_factory: TransactionAccountFactory,
    ) -> Result<AccountId, CreditLedgerError> {
        let id = AccountId::new();

        let mut op = cala.begin_operation().await?;
        account_factory
            .create_transaction_account_in_op(
                &mut op,
                id,
                &account_factory.control_sub_account.name,
                &account_factory.control_sub_account.name,
            )
            .await?;
        op.commit().await?;

        Ok(id)
    }

    async fn create_credit_omnibus_account(
        cala: &CalaLedger,
        account_factory: TransactionAccountFactory,
    ) -> Result<AccountId, CreditLedgerError> {
        let id = AccountId::new();

        let mut op = cala.begin_operation().await?;
        account_factory
            .create_transaction_account_in_op(
                &mut op,
                id,
                &account_factory.control_sub_account.name,
                &account_factory.control_sub_account.name,
            )
            .await?;
        op.commit().await?;

        Ok(id)
    }

    pub async fn create_credit_facility_control(
        cala: &CalaLedger,
    ) -> Result<VelocityControlId, CreditLedgerError> {
        let control = NewVelocityControl::builder()
            .id(CREDIT_FACILITY_VELOCITY_CONTROL_ID)
            .name("Credit Facility Control")
            .description("Velocity Control for Deposits")
            .build()
            .expect("build control");

        match cala.velocities().create_control(control).await {
            Err(cala_ledger::velocity::error::VelocityError::ControlIdAlreadyExists) => {
                Ok(CREDIT_FACILITY_VELOCITY_CONTROL_ID.into())
            }
            Err(e) => Err(e.into()),
            Ok(control) => Ok(control.id()),
        }
    }

    pub async fn add_credit_facility_control_to_account(
        &self,
        op: &mut cala_ledger::LedgerOperation<'_>,
        account_id: impl Into<AccountId>,
    ) -> Result<(), CreditLedgerError> {
        self.cala
            .velocities()
            .attach_control_to_account_in_op(
                op,
                self.credit_facility_control_id,
                account_id.into(),
                cala_ledger::tx_template::Params::default(),
            )
            .await?;
        Ok(())
    }

    pub async fn create_accounts_for_credit_facility(
        &self,
        op: &mut cala_ledger::LedgerOperation<'_>,
        credit_facility_id: CreditFacilityId,
        account_ids: CreditFacilityAccountIds,
    ) -> Result<(), CreditLedgerError> {
        let collateral_name = &format!(
            "Credit Facility Collateral Account for {}",
            credit_facility_id
        );
        self.account_factories
            .collateral
            .create_transaction_account_in_op(
                op,
                account_ids.collateral_account_id,
                collateral_name,
                collateral_name,
            )
            .await?;

        let facility_name = &format!(
            "Off-Balance-Sheet Facility Account for Credit Facility {}",
            credit_facility_id
        );
        self.account_factories
            .facility
            .create_transaction_account_in_op(
                op,
                account_ids.facility_account_id,
                facility_name,
                facility_name,
            )
            .await?;

        let disbursed_receivable_name = &format!(
            "Disbursed Receivable Account for Credit Facility {}",
            credit_facility_id
        );
        self.account_factories
            .disbursed_receivable
            .create_transaction_account_in_op(
                op,
                account_ids.disbursed_receivable_account_id,
                disbursed_receivable_name,
                disbursed_receivable_name,
            )
            .await?;

        let interest_receivable_name = &format!(
            "Interest Receivable Account for Credit Facility {}",
            credit_facility_id
        );
        self.account_factories
            .interest_receivable
            .create_transaction_account_in_op(
                op,
                account_ids.interest_receivable_account_id,
                interest_receivable_name,
                interest_receivable_name,
            )
            .await?;

        let interest_income_name = &format!(
            "Interest Income Account for Credit Facility {}",
            credit_facility_id
        );
        self.account_factories
            .interest_income
            .create_transaction_account_in_op(
                op,
                account_ids.interest_account_id,
                interest_income_name,
                interest_income_name,
            )
            .await?;

        let fee_income_name = &format!(
            "Fee Income Account for Credit Facility {}",
            credit_facility_id
        );
        self.account_factories
            .fee_income
            .create_transaction_account_in_op(
                op,
                account_ids.fee_income_account_id,
                fee_income_name,
                fee_income_name,
            )
            .await?;

        Ok(())
    }
}
