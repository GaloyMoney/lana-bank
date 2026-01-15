use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::instrument;
use tracing_macros::record_error_severity;

use audit::AuditInfo;
use core_accounting::{EntityRef, LedgerTransactionInitiator};
use es_entity::clock::ClockHandle;

mod deposit_accounts;
pub mod error;
mod templates;
mod velocity;

use cala_ledger::{
    CalaLedger, Currency, DebitOrCredit, JournalId, TransactionId,
    account::*,
    account_set::{AccountSet, AccountSetMemberId, AccountSetUpdate, NewAccountSet},
    tx_template::Params,
    velocity::{NewVelocityControl, VelocityControlId},
};

use crate::{
    DepositAccount, DepositAccountBalance, DepositReversalData, LedgerOmnibusAccountIds,
    WithdrawalReversalData,
    chart_of_accounts_integration::ChartOfAccountsIntegrationConfig,
    primitives::{
        CalaAccountId, CalaAccountSetId, DEPOSIT_ACCOUNT_ENTITY_TYPE, DepositAccountType,
        DepositId, UsdCents, WithdrawalId,
    },
};

pub(super) use deposit_accounts::*;
use error::*;

pub const DEPOSIT_INDIVIDUAL_ACCOUNT_SET_NAME: &str = "Deposit Individual Account Set";
pub const DEPOSIT_INDIVIDUAL_ACCOUNT_SET_REF: &str = "deposit-individual-account-set";
pub const DEPOSIT_GOVERNMENT_ENTITY_ACCOUNT_SET_NAME: &str =
    "Deposit Government Entity Account Set";
pub const DEPOSIT_GOVERNMENT_ENTITY_ACCOUNT_SET_REF: &str = "deposit-government-entity-account-set";
pub const DEPOSIT_PRIVATE_COMPANY_ACCOUNT_SET_NAME: &str = "Deposit Private Company Account Set";
pub const DEPOSIT_PRIVATE_COMPANY_ACCOUNT_SET_REF: &str = "deposit-private-company-account-set";
pub const DEPOSIT_BANK_ACCOUNT_SET_NAME: &str = "Deposit Bank Account Set";
pub const DEPOSIT_BANK_ACCOUNT_SET_REF: &str = "deposit-bank-account-set";
pub const DEPOSIT_FINANCIAL_INSTITUTION_ACCOUNT_SET_NAME: &str =
    "Deposit Financial Institution Account Set";
pub const DEPOSIT_FINANCIAL_INSTITUTION_ACCOUNT_SET_REF: &str =
    "deposit-financial-institution-account-set";
pub const DEPOSIT_NON_DOMICILED_INDIVIDUAL_ACCOUNT_SET_NAME: &str =
    "Deposit Non-Domiciled Company Account Set";
pub const DEPOSIT_NON_DOMICILED_INDIVIDUAL_ACCOUNT_SET_REF: &str =
    "deposit-non-domiciled-company-account-set";

pub const FROZEN_DEPOSIT_INDIVIDUAL_ACCOUNT_SET_NAME: &str =
    "Frozen Deposit Individual Account Set";
pub const FROZEN_DEPOSIT_INDIVIDUAL_ACCOUNT_SET_REF: &str = "frozen-deposit-individual-account-set";
pub const FROZEN_DEPOSIT_GOVERNMENT_ENTITY_ACCOUNT_SET_NAME: &str =
    "Frozen Deposit Government Entity Account Set";
pub const FROZEN_DEPOSIT_GOVERNMENT_ENTITY_ACCOUNT_SET_REF: &str =
    "frozen-deposit-government-entity-account-set";
pub const FROZEN_DEPOSIT_PRIVATE_COMPANY_ACCOUNT_SET_NAME: &str =
    "Frozen Deposit Private Company Account Set";
pub const FROZEN_DEPOSIT_PRIVATE_COMPANY_ACCOUNT_SET_REF: &str =
    "frozen-deposit-private-company-account-set";
pub const FROZEN_DEPOSIT_BANK_ACCOUNT_SET_NAME: &str = "Frozen Deposit Bank Account Set";
pub const FROZEN_DEPOSIT_BANK_ACCOUNT_SET_REF: &str = "frozen-deposit-bank-account-set";
pub const FROZEN_DEPOSIT_FINANCIAL_INSTITUTION_ACCOUNT_SET_NAME: &str =
    "Frozen Deposit Financial Institution Account Set";
pub const FROZEN_DEPOSIT_FINANCIAL_INSTITUTION_ACCOUNT_SET_REF: &str =
    "frozen-deposit-financial-institution-account-set";
pub const FROZEN_DEPOSIT_NON_DOMICILED_INDIVIDUAL_ACCOUNT_SET_NAME: &str =
    "Frozen Deposit Non-Domiciled Company Account Set";
pub const FROZEN_DEPOSIT_NON_DOMICILED_INDIVIDUAL_ACCOUNT_SET_REF: &str =
    "frozen-deposit-non-domiciled-company-account-set";

pub const DEPOSIT_OMNIBUS_ACCOUNT_SET_NAME: &str = "Deposit Omnibus Account Set";
pub const DEPOSIT_OMNIBUS_ACCOUNT_SET_REF: &str = "deposit-omnibus-account-set";
pub const DEPOSIT_OMNIBUS_ACCOUNT_REF: &str = "deposit-omnibus-account";

pub const DEPOSITS_VELOCITY_CONTROL_ID: uuid::Uuid =
    uuid::uuid!("00000000-0000-0000-0000-000000000001");

#[derive(Clone, Copy, Debug)]
pub struct InternalAccountSetDetails {
    id: CalaAccountSetId,
    normal_balance_type: DebitOrCredit,
}

#[derive(Clone, Copy)]
pub struct DepositAccountSets {
    individual: InternalAccountSetDetails,
    government_entity: InternalAccountSetDetails,
    private_company: InternalAccountSetDetails,
    bank: InternalAccountSetDetails,
    financial_institution: InternalAccountSetDetails,
    non_domiciled_individual: InternalAccountSetDetails,
}

impl DepositAccountSets {
    fn account_set_ids(&self) -> Vec<CalaAccountSetId> {
        vec![
            self.individual.id,
            self.government_entity.id,
            self.private_company.id,
            self.bank.id,
            self.financial_institution.id,
            self.non_domiciled_individual.id,
        ]
    }

    fn account_set_id_for_config(&self) -> CalaAccountSetId {
        self.individual.id
    }
}

#[derive(Clone)]
pub struct DepositLedger {
    cala: CalaLedger,
    clock: ClockHandle,
    journal_id: JournalId,
    deposit_account_sets: DepositAccountSets,
    frozen_deposit_account_sets: DepositAccountSets,
    deposit_omnibus_account_ids: LedgerOmnibusAccountIds,
    usd: Currency,
    deposit_control_id: VelocityControlId,
}

impl DepositLedger {
    #[record_error_severity]
    #[instrument(name = "deposit_ledger.init", skip_all)]
    pub async fn init(
        cala: &CalaLedger,
        journal_id: JournalId,
        clock: ClockHandle,
    ) -> Result<Self, DepositLedgerError> {
        templates::RecordDeposit::init(cala).await?;
        templates::InitiateWithdraw::init(cala).await?;
        templates::DenyWithdraw::init(cala).await?;
        templates::CancelWithdraw::init(cala).await?;
        templates::ConfirmWithdraw::init(cala).await?;
        templates::RevertWithdraw::init(cala).await?;
        templates::RevertDeposit::init(cala).await?;
        templates::FreezeAccount::init(cala).await?;
        templates::UnfreezeAccount::init(cala).await?;

        let deposits_normal_balance_type = DebitOrCredit::Credit;

        let individual_deposit_account_set_id = Self::find_or_create_account_set(
            cala,
            journal_id,
            format!("{journal_id}:{DEPOSIT_INDIVIDUAL_ACCOUNT_SET_REF}"),
            DEPOSIT_INDIVIDUAL_ACCOUNT_SET_NAME.to_string(),
            deposits_normal_balance_type,
        )
        .await?;

        let government_entity_deposit_account_set_id = Self::find_or_create_account_set(
            cala,
            journal_id,
            format!("{journal_id}:{DEPOSIT_GOVERNMENT_ENTITY_ACCOUNT_SET_REF}"),
            DEPOSIT_GOVERNMENT_ENTITY_ACCOUNT_SET_NAME.to_string(),
            deposits_normal_balance_type,
        )
        .await?;

        let private_company_deposit_account_set_id = Self::find_or_create_account_set(
            cala,
            journal_id,
            format!("{journal_id}:{DEPOSIT_PRIVATE_COMPANY_ACCOUNT_SET_REF}"),
            DEPOSIT_PRIVATE_COMPANY_ACCOUNT_SET_NAME.to_string(),
            deposits_normal_balance_type,
        )
        .await?;

        let bank_deposit_account_set_id = Self::find_or_create_account_set(
            cala,
            journal_id,
            format!("{journal_id}:{DEPOSIT_BANK_ACCOUNT_SET_REF}"),
            DEPOSIT_BANK_ACCOUNT_SET_NAME.to_string(),
            deposits_normal_balance_type,
        )
        .await?;

        let financial_institution_deposit_account_set_id = Self::find_or_create_account_set(
            cala,
            journal_id,
            format!("{journal_id}:{DEPOSIT_FINANCIAL_INSTITUTION_ACCOUNT_SET_REF}"),
            DEPOSIT_FINANCIAL_INSTITUTION_ACCOUNT_SET_NAME.to_string(),
            deposits_normal_balance_type,
        )
        .await?;

        let non_domiciled_company_deposit_account_set_id = Self::find_or_create_account_set(
            cala,
            journal_id,
            format!("{journal_id}:{DEPOSIT_NON_DOMICILED_INDIVIDUAL_ACCOUNT_SET_REF}"),
            DEPOSIT_NON_DOMICILED_INDIVIDUAL_ACCOUNT_SET_NAME.to_string(),
            deposits_normal_balance_type,
        )
        .await?;

        let frozen_individual_deposit_account_set_id = Self::find_or_create_account_set(
            cala,
            journal_id,
            format!("{journal_id}:{FROZEN_DEPOSIT_INDIVIDUAL_ACCOUNT_SET_REF}"),
            FROZEN_DEPOSIT_INDIVIDUAL_ACCOUNT_SET_NAME.to_string(),
            deposits_normal_balance_type,
        )
        .await?;

        let frozen_government_entity_deposit_account_set_id = Self::find_or_create_account_set(
            cala,
            journal_id,
            format!("{journal_id}:{FROZEN_DEPOSIT_GOVERNMENT_ENTITY_ACCOUNT_SET_REF}"),
            FROZEN_DEPOSIT_GOVERNMENT_ENTITY_ACCOUNT_SET_NAME.to_string(),
            deposits_normal_balance_type,
        )
        .await?;

        let frozen_private_company_deposit_account_set_id = Self::find_or_create_account_set(
            cala,
            journal_id,
            format!("{journal_id}:{FROZEN_DEPOSIT_PRIVATE_COMPANY_ACCOUNT_SET_REF}"),
            FROZEN_DEPOSIT_PRIVATE_COMPANY_ACCOUNT_SET_NAME.to_string(),
            deposits_normal_balance_type,
        )
        .await?;

        let frozen_bank_deposit_account_set_id = Self::find_or_create_account_set(
            cala,
            journal_id,
            format!("{journal_id}:{FROZEN_DEPOSIT_BANK_ACCOUNT_SET_REF}"),
            FROZEN_DEPOSIT_BANK_ACCOUNT_SET_NAME.to_string(),
            deposits_normal_balance_type,
        )
        .await?;

        let frozen_financial_institution_deposit_account_set_id = Self::find_or_create_account_set(
            cala,
            journal_id,
            format!("{journal_id}:{FROZEN_DEPOSIT_FINANCIAL_INSTITUTION_ACCOUNT_SET_REF}"),
            FROZEN_DEPOSIT_FINANCIAL_INSTITUTION_ACCOUNT_SET_NAME.to_string(),
            deposits_normal_balance_type,
        )
        .await?;

        let frozen_non_domiciled_company_deposit_account_set_id = Self::find_or_create_account_set(
            cala,
            journal_id,
            format!("{journal_id}:{FROZEN_DEPOSIT_NON_DOMICILED_INDIVIDUAL_ACCOUNT_SET_REF}"),
            FROZEN_DEPOSIT_NON_DOMICILED_INDIVIDUAL_ACCOUNT_SET_NAME.to_string(),
            deposits_normal_balance_type,
        )
        .await?;

        let deposit_omnibus_account_ids = Self::find_or_create_omnibus_account(
            cala,
            journal_id,
            format!("{journal_id}:{DEPOSIT_OMNIBUS_ACCOUNT_SET_REF}"),
            format!("{journal_id}:{DEPOSIT_OMNIBUS_ACCOUNT_REF}"),
            DEPOSIT_OMNIBUS_ACCOUNT_SET_NAME.to_string(),
            DebitOrCredit::Debit,
        )
        .await?;

        let overdraft_prevention_id = velocity::OverdraftPrevention::init(cala).await?;

        let deposit_control_id = Self::create_deposit_control(cala).await?;

        match cala
            .velocities()
            .add_limit_to_control(deposit_control_id, overdraft_prevention_id)
            .await
        {
            Ok(_)
            | Err(cala_ledger::velocity::error::VelocityError::LimitAlreadyAddedToControl) => {}
            Err(e) => return Err(e.into()),
        }
        Ok(Self {
            clock,
            cala: cala.clone(),
            journal_id,
            deposit_account_sets: DepositAccountSets {
                individual: InternalAccountSetDetails {
                    id: individual_deposit_account_set_id,
                    normal_balance_type: deposits_normal_balance_type,
                },
                government_entity: InternalAccountSetDetails {
                    id: government_entity_deposit_account_set_id,
                    normal_balance_type: deposits_normal_balance_type,
                },
                private_company: InternalAccountSetDetails {
                    id: private_company_deposit_account_set_id,
                    normal_balance_type: deposits_normal_balance_type,
                },
                bank: InternalAccountSetDetails {
                    id: bank_deposit_account_set_id,
                    normal_balance_type: deposits_normal_balance_type,
                },
                financial_institution: InternalAccountSetDetails {
                    id: financial_institution_deposit_account_set_id,
                    normal_balance_type: deposits_normal_balance_type,
                },
                non_domiciled_individual: InternalAccountSetDetails {
                    id: non_domiciled_company_deposit_account_set_id,
                    normal_balance_type: deposits_normal_balance_type,
                },
            },
            frozen_deposit_account_sets: DepositAccountSets {
                individual: InternalAccountSetDetails {
                    id: frozen_individual_deposit_account_set_id,
                    normal_balance_type: deposits_normal_balance_type,
                },
                government_entity: InternalAccountSetDetails {
                    id: frozen_government_entity_deposit_account_set_id,
                    normal_balance_type: deposits_normal_balance_type,
                },
                private_company: InternalAccountSetDetails {
                    id: frozen_private_company_deposit_account_set_id,
                    normal_balance_type: deposits_normal_balance_type,
                },
                bank: InternalAccountSetDetails {
                    id: frozen_bank_deposit_account_set_id,
                    normal_balance_type: deposits_normal_balance_type,
                },
                financial_institution: InternalAccountSetDetails {
                    id: frozen_financial_institution_deposit_account_set_id,
                    normal_balance_type: deposits_normal_balance_type,
                },
                non_domiciled_individual: InternalAccountSetDetails {
                    id: frozen_non_domiciled_company_deposit_account_set_id,
                    normal_balance_type: deposits_normal_balance_type,
                },
            },
            deposit_omnibus_account_ids,
            deposit_control_id,
            usd: Currency::USD,
        })
    }

    #[record_error_severity]
    #[instrument(name = "deposit_ledger.find_or_create_account_set", skip(cala, name), fields(journal_id = %journal_id, reference = %reference, account_set_name = %name))]
    async fn find_or_create_account_set(
        cala: &CalaLedger,
        journal_id: JournalId,
        reference: String,
        name: String,
        normal_balance_type: DebitOrCredit,
    ) -> Result<CalaAccountSetId, DepositLedgerError> {
        match cala
            .account_sets()
            .find_by_external_id(reference.to_string())
            .await
        {
            Ok(account_set) if account_set.values().journal_id != journal_id => {
                return Err(DepositLedgerError::JournalIdMismatch);
            }
            Ok(account_set) => return Ok(account_set.id),
            Err(e) if e.was_not_found() => (),
            Err(e) => return Err(e.into()),
        };

        let id = CalaAccountSetId::new();
        let new_account_set = NewAccountSet::builder()
            .id(id)
            .journal_id(journal_id)
            .external_id(reference.to_string())
            .name(name.clone())
            .description(name)
            .normal_balance_type(normal_balance_type)
            .build()
            .expect("Could not build new account set");
        match cala.account_sets().create(new_account_set).await {
            Ok(set) => Ok(set.id),
            Err(cala_ledger::account_set::error::AccountSetError::ExternalIdAlreadyExists) => {
                Ok(cala.account_sets().find_by_external_id(reference).await?.id)
            }

            Err(e) => Err(e.into()),
        }
    }

    #[record_error_severity]
    #[instrument(name = "deposit_ledger.find_or_create_omnibus_account", skip(cala, name), fields(journal_id = %journal_id, reference = %reference, account_set_name = %name))]
    async fn find_or_create_omnibus_account(
        cala: &CalaLedger,
        journal_id: JournalId,
        account_set_reference: String,
        reference: String,
        name: String,
        normal_balance_type: DebitOrCredit,
    ) -> Result<LedgerOmnibusAccountIds, DepositLedgerError> {
        let account_set_id = Self::find_or_create_account_set(
            cala,
            journal_id,
            account_set_reference,
            name.to_string(),
            normal_balance_type,
        )
        .await?;

        let members = cala
            .account_sets()
            .list_members_by_created_at(account_set_id, Default::default())
            .await?
            .entities;
        if !members.is_empty() {
            match members[0].id {
                AccountSetMemberId::Account(id) => {
                    return Ok(LedgerOmnibusAccountIds {
                        account_set_id,
                        account_id: id,
                    });
                }
                AccountSetMemberId::AccountSet(_) => {
                    return Err(DepositLedgerError::NonAccountMemberFoundInAccountSet(
                        account_set_id.to_string(),
                    ));
                }
            }
        }

        let mut op = cala.begin_operation().await?;
        let id = CalaAccountId::new();
        let new_ledger_account = NewAccount::builder()
            .id(id)
            .external_id(reference.to_string())
            .name(name.clone())
            .description(name)
            .code(id.to_string())
            .normal_balance_type(normal_balance_type)
            .build()
            .expect("Could not build new account");

        let account_id = match cala
            .accounts()
            .create_in_op(&mut op, new_ledger_account)
            .await
        {
            Ok(account) => {
                cala.account_sets()
                    .add_member_in_op(&mut op, account_set_id, account.id)
                    .await?;

                op.commit().await?;

                id
            }
            Err(cala_ledger::account::error::AccountError::ExternalIdAlreadyExists) => {
                cala.accounts().find_by_external_id(reference).await?.id
            }
            Err(e) => return Err(e.into()),
        };

        Ok(LedgerOmnibusAccountIds {
            account_set_id,
            account_id,
        })
    }

    #[record_error_severity]
    #[instrument(name = "deposit_ledger.account_history", skip_all, fields(account_id = tracing::field::Empty))]
    pub async fn account_history<T, U>(
        &self,
        id: impl Into<AccountId>,
        cursor: es_entity::PaginatedQueryArgs<U>,
    ) -> Result<es_entity::PaginatedQueryRet<T, U>, DepositLedgerError>
    where
        T: From<cala_ledger::entry::Entry>,
        U: std::fmt::Debug + From<cala_ledger::entry::EntriesByCreatedAtCursor>,
        cala_ledger::entry::EntriesByCreatedAtCursor: From<U>,
    {
        let id = id.into();
        tracing::Span::current().record("account_id", tracing::field::debug(&id));

        let cala_cursor = es_entity::PaginatedQueryArgs {
            after: cursor
                .after
                .map(cala_ledger::entry::EntriesByCreatedAtCursor::from),
            first: cursor.first,
        };

        let ret = self
            .cala
            .entries()
            .list_for_account_id(id, cala_cursor, es_entity::ListDirection::Descending)
            .await?;
        let entities = ret.entities.into_iter().map(T::from).collect();
        Ok(es_entity::PaginatedQueryRet {
            entities,
            has_next_page: ret.has_next_page,
            end_cursor: ret.end_cursor.map(U::from),
        })
    }

    #[record_error_severity]
    #[instrument(
        name = "deposit_ledger.record_deposit",
        skip_all,
        fields(entity_id = tracing::field::Empty, credit_account_id = tracing::field::Empty)
    )]
    pub async fn record_deposit(
        &self,
        op: &mut es_entity::DbOp<'_>,
        entity_id: DepositId,
        amount: UsdCents,
        credit_account_id: impl Into<AccountId>,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<(), DepositLedgerError> {
        let tx_id = entity_id.into();
        tracing::Span::current().record("entity_id", tracing::field::debug(&entity_id));
        let credit_account_id = credit_account_id.into();
        tracing::Span::current().record(
            "credit_account_id",
            tracing::field::debug(&credit_account_id),
        );

        let params = templates::RecordDepositParams {
            entity_id: entity_id.into(),
            journal_id: self.journal_id,
            currency: self.usd,
            amount: amount.to_usd(),
            deposit_omnibus_account_id: self.deposit_omnibus_account_ids.account_id,
            credit_account_id,
            initiated_by,
            effective_date: self.clock.today(),
        };
        self.cala
            .post_transaction_in_op(op, tx_id, templates::RECORD_DEPOSIT_CODE, params)
            .await?;
        Ok(())
    }

    #[record_error_severity]
    #[instrument(
        name = "deposit_ledger.initiate_withdrawal",
        skip_all,
        fields(entity_id = tracing::field::Empty, credit_account_id = tracing::field::Empty)
    )]
    pub async fn initiate_withdrawal(
        &self,
        op: &mut es_entity::DbOp<'_>,
        entity_id: WithdrawalId,
        amount: UsdCents,
        credit_account_id: impl Into<AccountId>,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<(), DepositLedgerError> {
        let tx_id = entity_id.into();
        tracing::Span::current().record("entity_id", tracing::field::debug(&entity_id));
        let credit_account_id = credit_account_id.into();
        tracing::Span::current().record(
            "credit_account_id",
            tracing::field::debug(&credit_account_id),
        );

        let params = templates::InitiateWithdrawParams {
            entity_id: entity_id.into(),
            journal_id: self.journal_id,
            deposit_omnibus_account_id: self.deposit_omnibus_account_ids.account_id,
            credit_account_id,
            amount: amount.to_usd(),
            currency: self.usd,
            initiated_by,
            effective_date: self.clock.today(),
        };

        self.cala
            .post_transaction_in_op(op, tx_id, templates::INITIATE_WITHDRAW_CODE, params)
            .await?;

        Ok(())
    }

    #[record_error_severity]
    #[instrument(
        name = "deposit_ledger.deny_withdrawal",
        skip_all,
        fields(entity_id = tracing::field::Empty, credit_account_id = tracing::field::Empty)
    )]
    pub async fn deny_withdrawal(
        &self,
        op: &mut es_entity::DbOp<'_>,
        entity_id: WithdrawalId,
        tx_id: impl Into<TransactionId>,
        amount: UsdCents,
        credit_account_id: impl Into<AccountId>,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<(), DepositLedgerError> {
        let tx_id = tx_id.into();
        tracing::Span::current().record("entity_id", tracing::field::debug(&entity_id));
        tracing::Span::current().record("tx_id", tracing::field::debug(&tx_id));
        let credit_account_id = credit_account_id.into();
        tracing::Span::current().record(
            "credit_account_id",
            tracing::field::debug(&credit_account_id),
        );

        let params = templates::DenyWithdrawParams {
            entity_id: entity_id.into(),
            journal_id: self.journal_id,
            deposit_omnibus_account_id: self.deposit_omnibus_account_ids.account_id,
            credit_account_id,
            amount: amount.to_usd(),
            currency: self.usd,
            initiated_by,
            effective_date: self.clock.today(),
        };

        self.cala
            .post_transaction_in_op(op, tx_id, templates::DENY_WITHDRAW_CODE, params)
            .await?;

        Ok(())
    }

    #[record_error_severity]
    #[instrument(name = "deposit_ledger.revert_withdrawal", skip(self, op))]
    pub async fn revert_withdrawal(
        &self,
        op: &mut es_entity::DbOp<'_>,
        reversal_data: WithdrawalReversalData,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<(), DepositLedgerError> {
        let params = templates::RevertWithdrawParams {
            entity_id: reversal_data.entity_id.into(),
            journal_id: self.journal_id,
            deposit_omnibus_account_id: self.deposit_omnibus_account_ids.account_id,
            credit_account_id: reversal_data.credit_account_id.into(),
            amount: reversal_data.amount.to_usd(),
            currency: self.usd,
            correlation_id: reversal_data.correlation_id,
            external_id: reversal_data.external_id,
            initiated_by,
            effective_date: self.clock.today(),
        };

        self.cala
            .post_transaction_in_op(
                op,
                reversal_data.ledger_tx_id,
                templates::REVERT_WITHDRAW_CODE,
                params,
            )
            .await?;

        Ok(())
    }

    #[record_error_severity]
    #[instrument(name = "deposit_ledger.revert_deposit", skip_all)]
    pub async fn revert_deposit(
        &self,
        op: &mut es_entity::DbOp<'_>,
        reversal_data: DepositReversalData,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<(), DepositLedgerError> {
        let params = templates::RevertDepositParams {
            entity_id: reversal_data.entity_id.into(),
            journal_id: self.journal_id,
            deposit_omnibus_account_id: self.deposit_omnibus_account_ids.account_id,
            credit_account_id: reversal_data.credit_account_id.into(),
            correlation_id: reversal_data.correlation_id,
            external_id: reversal_data.external_id,
            amount: reversal_data.amount.to_usd(),
            currency: self.usd,
            initiated_by,
            effective_date: self.clock.today(),
        };

        self.cala
            .post_transaction_in_op(
                op,
                reversal_data.ledger_tx_id,
                templates::REVERT_DEPOSIT_CODE,
                params,
            )
            .await?;

        Ok(())
    }

    #[record_error_severity]
    #[instrument(
        name = "deposit_ledger.freeze_account_in_op",
        skip(self, op, account),
        fields(
            account_id = %account.id,
            frozen_deposit_account_id = %account.account_ids.frozen_deposit_account_id,
            account_holder_id = %account.account_holder_id,
        )
    )]
    pub async fn freeze_account_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        account: &DepositAccount,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<(), DepositLedgerError> {
        let balance = self.balance(account.id).await?;

        if !balance.settled.is_zero() {
            let params = templates::FreezeAccountParams {
                journal_id: self.journal_id,
                account_id: account.account_ids.deposit_account_id,
                frozen_accounts_account_id: account.account_ids.frozen_deposit_account_id,
                amount: balance.settled.to_usd(),
                currency: self.usd,
                initiated_by,
                effective_date: self.clock.today(),
            };

            self.cala
                .post_transaction_in_op(
                    op,
                    TransactionId::new(),
                    templates::FREEZE_ACCOUNT_CODE,
                    params,
                )
                .await?;
        }

        self.cala
            .accounts()
            .lock_in_op(op, account.account_ids.deposit_account_id)
            .await?;

        Ok(())
    }

    #[record_error_severity]
    #[instrument(
      name = "deposit_ledger.unfreeze_account_in_op",
      skip(self, op, account),
      fields(
        account_id = %account.id,
        frozen_deposit_account_id = %account.account_ids.frozen_deposit_account_id,
        account_holder_id = %account.account_holder_id,
      )
  )]
    pub async fn unfreeze_account_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        account: &DepositAccount,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<(), DepositLedgerError> {
        let frozen_balance = self
            .balance(account.account_ids.frozen_deposit_account_id)
            .await?;

        self.cala
            .accounts()
            .unlock_in_op(op, account.account_ids.deposit_account_id)
            .await?;

        if !frozen_balance.settled.is_zero() {
            let params = templates::UnfreezeAccountParams {
                journal_id: self.journal_id,
                account_id: account.account_ids.deposit_account_id,
                frozen_accounts_account_id: account.account_ids.frozen_deposit_account_id,
                amount: frozen_balance.settled.to_usd(),
                currency: self.usd,
                initiated_by,
                effective_date: self.clock.today(),
            };

            self.cala
                .post_transaction_in_op(
                    op,
                    TransactionId::new(),
                    templates::UNFREEZE_ACCOUNT_CODE,
                    params,
                )
                .await?;
        }

        Ok(())
    }

    #[record_error_severity]
    #[instrument(name = "deposit_ledger.lock_account", skip(self, op))]
    pub async fn lock_account(
        &self,
        op: &mut es_entity::DbOp<'_>,
        account_id: AccountId,
    ) -> Result<(), DepositLedgerError> {
        self.cala.accounts().lock_in_op(op, account_id).await?;

        Ok(())
    }

    #[record_error_severity]
    #[instrument(
        name = "deposit_ledger.confirm_withdrawal",
        skip_all,
        fields(entity_id = tracing::field::Empty, tx_id = tracing::field::Empty, credit_account_id = tracing::field::Empty)
    )]
    pub async fn confirm_withdrawal(
        &self,
        op: &mut es_entity::DbOp<'_>,
        entity_id: WithdrawalId,
        tx_id: impl Into<TransactionId>,
        correlation_id: String,
        amount: UsdCents,
        credit_account_id: impl Into<AccountId>,
        external_id: String,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<(), DepositLedgerError> {
        let tx_id = tx_id.into();
        tracing::Span::current().record("entity_id", tracing::field::debug(&entity_id));
        tracing::Span::current().record("tx_id", tracing::field::debug(&tx_id));
        let credit_account_id = credit_account_id.into();
        tracing::Span::current().record(
            "credit_account_id",
            tracing::field::debug(&credit_account_id),
        );

        let params = templates::ConfirmWithdrawParams {
            entity_id: entity_id.into(),
            journal_id: self.journal_id,
            currency: self.usd,
            amount: amount.to_usd(),
            deposit_omnibus_account_id: self.deposit_omnibus_account_ids.account_id,
            credit_account_id,
            correlation_id,
            external_id,
            initiated_by,
            effective_date: self.clock.today(),
        };

        self.cala
            .post_transaction_in_op(op, tx_id, templates::CONFIRM_WITHDRAW_CODE, params)
            .await?;
        Ok(())
    }

    #[record_error_severity]
    #[instrument(
        name = "deposit_ledger.cancel_withdrawal",
        skip_all,
        fields(entity_id = tracing::field::Empty, tx_id = tracing::field::Empty, credit_account_id = tracing::field::Empty)
    )]
    pub async fn cancel_withdrawal(
        &self,
        op: &mut es_entity::DbOp<'_>,
        entity_id: WithdrawalId,
        tx_id: impl Into<TransactionId>,
        amount: UsdCents,
        credit_account_id: impl Into<AccountId>,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<(), DepositLedgerError> {
        let tx_id = tx_id.into();
        tracing::Span::current().record("entity_id", tracing::field::debug(&entity_id));
        tracing::Span::current().record("tx_id", tracing::field::debug(&tx_id));
        let credit_account_id = credit_account_id.into();
        tracing::Span::current().record(
            "credit_account_id",
            tracing::field::debug(&credit_account_id),
        );

        let params = templates::CancelWithdrawParams {
            entity_id: entity_id.into(),
            journal_id: self.journal_id,
            currency: self.usd,
            amount: amount.to_usd(),
            credit_account_id,
            deposit_omnibus_account_id: self.deposit_omnibus_account_ids.account_id,
            initiated_by,
            effective_date: self.clock.today(),
        };

        self.cala
            .post_transaction_in_op(op, tx_id, templates::CANCEL_WITHDRAW_CODE, params)
            .await?;
        Ok(())
    }

    #[record_error_severity]
    #[instrument(name = "deposit_ledger.balance", skip_all, fields(account_id = tracing::field::Empty))]
    pub async fn balance(
        &self,
        account_id: impl Into<AccountId>,
    ) -> Result<DepositAccountBalance, DepositLedgerError> {
        let account_id = account_id.into();
        tracing::Span::current().record("account_id", tracing::field::debug(&account_id));
        match self
            .cala
            .balances()
            .find(self.journal_id, account_id, self.usd)
            .await
        {
            Ok(balances) => Ok(DepositAccountBalance {
                settled: UsdCents::try_from_usd(balances.settled())?,
                pending: UsdCents::try_from_usd(balances.pending())?,
            }),
            Err(cala_ledger::balance::error::BalanceError::NotFound(..)) => {
                Ok(DepositAccountBalance::ZERO)
            }
            Err(e) => Err(e.into()),
        }
    }

    #[record_error_severity]
    #[instrument(name = "deposit_ledger.create_deposit_accounts", skip_all)]
    pub async fn create_deposit_accounts(
        &self,
        op: &mut es_entity::DbOp<'_>,
        account: &DepositAccount,
        deposit_account_type: impl Into<DepositAccountType>,
    ) -> Result<(), DepositLedgerError> {
        let holder_id = account.account_holder_id;
        let deposit_account_type = deposit_account_type.into();

        let entity_ref = EntityRef::new(DEPOSIT_ACCOUNT_ENTITY_TYPE, account.id);
        let deposit_account_name = format!("Deposit Account {holder_id}");
        self.create_account_in_op(
            op,
            account.id,
            self.deposit_internal_account_set_from_type(deposit_account_type),
            &format!("deposit-customer-account:{holder_id}"),
            &deposit_account_name,
            &deposit_account_name,
            entity_ref.clone(),
        )
        .await?;

        self.add_deposit_control_to_account(op, account.id).await?;

        let frozen_deposit_account_name = format!("Frozen Deposit Account {holder_id}");
        self.create_account_in_op(
            op,
            account.account_ids.frozen_deposit_account_id,
            self.frozen_deposit_internal_account_set_from_type(deposit_account_type),
            &format!("frozen-deposit-customer-account:{holder_id}"),
            &frozen_deposit_account_name,
            &frozen_deposit_account_name,
            entity_ref,
        )
        .await?;

        Ok(())
    }

    fn deposit_internal_account_set_from_type(
        &self,
        deposit_account_type: DepositAccountType,
    ) -> InternalAccountSetDetails {
        match deposit_account_type {
            DepositAccountType::Individual => self.deposit_account_sets.individual,
            DepositAccountType::GovernmentEntity => self.deposit_account_sets.government_entity,
            DepositAccountType::PrivateCompany => self.deposit_account_sets.private_company,
            DepositAccountType::Bank => self.deposit_account_sets.bank,
            DepositAccountType::FinancialInstitution => {
                self.deposit_account_sets.financial_institution
            }
            DepositAccountType::NonDomiciledCompany => {
                self.deposit_account_sets.non_domiciled_individual
            }
        }
    }

    fn frozen_deposit_internal_account_set_from_type(
        &self,
        deposit_account_type: DepositAccountType,
    ) -> InternalAccountSetDetails {
        match deposit_account_type {
            DepositAccountType::Individual => self.frozen_deposit_account_sets.individual,
            DepositAccountType::GovernmentEntity => {
                self.frozen_deposit_account_sets.government_entity
            }
            DepositAccountType::PrivateCompany => self.frozen_deposit_account_sets.private_company,
            DepositAccountType::Bank => self.frozen_deposit_account_sets.bank,
            DepositAccountType::FinancialInstitution => {
                self.frozen_deposit_account_sets.financial_institution
            }
            DepositAccountType::NonDomiciledCompany => {
                self.frozen_deposit_account_sets.non_domiciled_individual
            }
        }
    }

    #[record_error_severity]
    #[instrument(
        name = "deposit_ledger.create_account_in_op",
        skip_all,
        fields(account_id = tracing::field::Empty)
    )]
    async fn create_account_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        id: impl Into<CalaAccountId>,
        parent_account_set: InternalAccountSetDetails,
        reference: &str,
        name: &str,
        description: &str,
        entity_ref: core_accounting::EntityRef,
    ) -> Result<(), DepositLedgerError> {
        let id = id.into();
        tracing::Span::current().record("account_id", tracing::field::debug(&id));

        let new_ledger_account = NewAccount::builder()
            .id(id)
            .external_id(reference)
            .name(name)
            .description(description)
            .code(id.to_string())
            .normal_balance_type(parent_account_set.normal_balance_type)
            .metadata(serde_json::json!({"entity_ref": entity_ref}))
            .expect("Could not add metadata")
            .build()
            .expect("Could not build new account");
        let ledger_account = self
            .cala
            .accounts()
            .create_in_op(op, new_ledger_account)
            .await?;
        self.cala
            .account_sets()
            .add_member_in_op(op, parent_account_set.id, ledger_account.id)
            .await?;

        Ok(())
    }

    #[record_error_severity]
    #[instrument(name = "deposit_ledger.create_deposit_control", skip(cala))]
    pub async fn create_deposit_control(
        cala: &CalaLedger,
    ) -> Result<VelocityControlId, DepositLedgerError> {
        let control = NewVelocityControl::builder()
            .id(DEPOSITS_VELOCITY_CONTROL_ID)
            .name("Deposit Control")
            .description("Velocity Control for Deposits")
            .build()
            .expect("build control");

        match cala.velocities().create_control(control).await {
            Err(cala_ledger::velocity::error::VelocityError::ControlIdAlreadyExists) => {
                Ok(DEPOSITS_VELOCITY_CONTROL_ID.into())
            }
            Err(e) => Err(e.into()),
            Ok(control) => Ok(control.id()),
        }
    }

    #[record_error_severity]
    #[instrument(
        name = "deposit_ledger.add_deposit_control_to_account",
        skip_all,
        fields(account_id = tracing::field::Empty)
    )]
    pub async fn add_deposit_control_to_account(
        &self,
        op: &mut es_entity::DbOp<'_>,
        account_id: impl Into<AccountId>,
    ) -> Result<(), DepositLedgerError> {
        let account_id = account_id.into();
        tracing::Span::current().record("account_id", tracing::field::debug(&account_id));
        self.cala
            .velocities()
            .attach_control_to_account_in_op(
                op,
                self.deposit_control_id,
                account_id,
                Params::default(),
            )
            .await?;

        Ok(())
    }

    #[record_error_severity]
    #[instrument(
        name = "deposit_ledger.get_chart_of_accounts_integration_config",
        skip(self)
    )]
    pub async fn get_chart_of_accounts_integration_config(
        &self,
    ) -> Result<Option<ChartOfAccountsIntegrationConfig>, DepositLedgerError> {
        let account_set = self
            .cala
            .account_sets()
            .find(self.deposit_account_sets.account_set_id_for_config())
            .await?;
        if let Some(meta) = account_set.values().metadata.as_ref() {
            let meta: ChartOfAccountsIntegrationMeta =
                serde_json::from_value(meta.clone()).expect("Could not deserialize metadata");
            Ok(Some(meta.config))
        } else {
            Ok(None)
        }
    }

    #[record_error_severity]
    #[instrument(name = "deposit_ledger.attach_charts_account_set", skip_all)]
    async fn attach_charts_account_set<F>(
        &self,
        op: &mut es_entity::DbOpWithTime<'_>,
        account_sets: &mut HashMap<CalaAccountSetId, AccountSet>,
        internal_account_set_id: CalaAccountSetId,
        parent_account_set_id: CalaAccountSetId,
        new_meta: &ChartOfAccountsIntegrationMeta,
        old_parent_id_getter: F,
    ) -> Result<(), DepositLedgerError>
    where
        F: FnOnce(ChartOfAccountsIntegrationMeta) -> CalaAccountSetId,
    {
        let mut internal_account_set = account_sets
            .remove(&internal_account_set_id)
            .expect("internal account set not found");

        if let Some(old_meta) = internal_account_set.values().metadata.as_ref() {
            let old_meta: ChartOfAccountsIntegrationMeta =
                serde_json::from_value(old_meta.clone()).expect("Could not deserialize metadata");
            let old_parent_account_set_id = old_parent_id_getter(old_meta);
            if old_parent_account_set_id != parent_account_set_id {
                self.cala
                    .account_sets()
                    .remove_member_in_op(op, old_parent_account_set_id, internal_account_set_id)
                    .await?;
            }
        }

        self.cala
            .account_sets()
            .add_member_in_op(op, parent_account_set_id, internal_account_set_id)
            .await?;
        let mut update = AccountSetUpdate::default();
        update
            .metadata(new_meta)
            .expect("Could not update metadata");
        if internal_account_set.update(update).did_execute() {
            self.cala
                .account_sets()
                .persist_in_op(op, &mut internal_account_set)
                .await?;
        }

        Ok(())
    }

    #[record_error_severity]
    #[instrument(
        name = "deposit_ledger.attach_chart_of_accounts_account_sets",
        skip_all
    )]
    pub async fn attach_chart_of_accounts_account_sets(
        &self,
        charts_integration_meta: ChartOfAccountsIntegrationMeta,
    ) -> Result<(), DepositLedgerError> {
        let mut op = self.cala.begin_operation().await?;

        let mut account_set_ids = vec![self.deposit_omnibus_account_ids.account_set_id];
        account_set_ids.extend(self.deposit_account_sets.account_set_ids());
        account_set_ids.extend(self.frozen_deposit_account_sets.account_set_ids());

        let mut account_sets = self
            .cala
            .account_sets()
            .find_all_in_op::<AccountSet>(&mut op, &account_set_ids)
            .await?;

        let ChartOfAccountsIntegrationMeta {
            config: _,
            audit_info: _,
            omnibus_parent_account_set_id,
            individual_deposit_accounts_parent_account_set_id,
            government_entity_deposit_accounts_parent_account_set_id,
            private_company_deposit_accounts_parent_account_set_id,
            bank_deposit_accounts_parent_account_set_id,
            financial_institution_deposit_accounts_parent_account_set_id,
            non_domiciled_individual_deposit_accounts_parent_account_set_id,
            frozen_individual_deposit_accounts_parent_account_set_id,
            frozen_government_entity_deposit_accounts_parent_account_set_id,
            frozen_private_company_deposit_accounts_parent_account_set_id,
            frozen_bank_deposit_accounts_parent_account_set_id,
            frozen_financial_institution_deposit_accounts_parent_account_set_id,
            frozen_non_domiciled_individual_deposit_accounts_parent_account_set_id,
        } = &charts_integration_meta;

        self.attach_charts_account_set(
            &mut op,
            &mut account_sets,
            self.deposit_omnibus_account_ids.account_set_id,
            *omnibus_parent_account_set_id,
            &charts_integration_meta,
            |meta| meta.omnibus_parent_account_set_id,
        )
        .await?;

        self.attach_charts_account_set(
            &mut op,
            &mut account_sets,
            self.deposit_account_sets.individual.id,
            *individual_deposit_accounts_parent_account_set_id,
            &charts_integration_meta,
            |meta| meta.individual_deposit_accounts_parent_account_set_id,
        )
        .await?;

        self.attach_charts_account_set(
            &mut op,
            &mut account_sets,
            self.deposit_account_sets.government_entity.id,
            *government_entity_deposit_accounts_parent_account_set_id,
            &charts_integration_meta,
            |meta| meta.government_entity_deposit_accounts_parent_account_set_id,
        )
        .await?;

        self.attach_charts_account_set(
            &mut op,
            &mut account_sets,
            self.deposit_account_sets.private_company.id,
            *private_company_deposit_accounts_parent_account_set_id,
            &charts_integration_meta,
            |meta| meta.private_company_deposit_accounts_parent_account_set_id,
        )
        .await?;

        self.attach_charts_account_set(
            &mut op,
            &mut account_sets,
            self.deposit_account_sets.bank.id,
            *bank_deposit_accounts_parent_account_set_id,
            &charts_integration_meta,
            |meta| meta.bank_deposit_accounts_parent_account_set_id,
        )
        .await?;

        self.attach_charts_account_set(
            &mut op,
            &mut account_sets,
            self.deposit_account_sets.financial_institution.id,
            *financial_institution_deposit_accounts_parent_account_set_id,
            &charts_integration_meta,
            |meta| meta.financial_institution_deposit_accounts_parent_account_set_id,
        )
        .await?;

        self.attach_charts_account_set(
            &mut op,
            &mut account_sets,
            self.deposit_account_sets.non_domiciled_individual.id,
            *non_domiciled_individual_deposit_accounts_parent_account_set_id,
            &charts_integration_meta,
            |meta| meta.non_domiciled_individual_deposit_accounts_parent_account_set_id,
        )
        .await?;

        self.attach_charts_account_set(
            &mut op,
            &mut account_sets,
            self.frozen_deposit_account_sets.individual.id,
            *frozen_individual_deposit_accounts_parent_account_set_id,
            &charts_integration_meta,
            |meta| meta.frozen_individual_deposit_accounts_parent_account_set_id,
        )
        .await?;

        self.attach_charts_account_set(
            &mut op,
            &mut account_sets,
            self.frozen_deposit_account_sets.government_entity.id,
            *frozen_government_entity_deposit_accounts_parent_account_set_id,
            &charts_integration_meta,
            |meta| meta.frozen_government_entity_deposit_accounts_parent_account_set_id,
        )
        .await?;

        self.attach_charts_account_set(
            &mut op,
            &mut account_sets,
            self.frozen_deposit_account_sets.private_company.id,
            *frozen_private_company_deposit_accounts_parent_account_set_id,
            &charts_integration_meta,
            |meta| meta.frozen_private_company_deposit_accounts_parent_account_set_id,
        )
        .await?;

        self.attach_charts_account_set(
            &mut op,
            &mut account_sets,
            self.frozen_deposit_account_sets.bank.id,
            *frozen_bank_deposit_accounts_parent_account_set_id,
            &charts_integration_meta,
            |meta| meta.frozen_bank_deposit_accounts_parent_account_set_id,
        )
        .await?;

        self.attach_charts_account_set(
            &mut op,
            &mut account_sets,
            self.frozen_deposit_account_sets.financial_institution.id,
            *frozen_financial_institution_deposit_accounts_parent_account_set_id,
            &charts_integration_meta,
            |meta| meta.frozen_financial_institution_deposit_accounts_parent_account_set_id,
        )
        .await?;

        self.attach_charts_account_set(
            &mut op,
            &mut account_sets,
            self.frozen_deposit_account_sets.non_domiciled_individual.id,
            *frozen_non_domiciled_individual_deposit_accounts_parent_account_set_id,
            &charts_integration_meta,
            |meta| meta.frozen_non_domiciled_individual_deposit_accounts_parent_account_set_id,
        )
        .await?;

        op.commit().await?;

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChartOfAccountsIntegrationMeta {
    pub config: ChartOfAccountsIntegrationConfig,
    pub audit_info: AuditInfo,

    pub omnibus_parent_account_set_id: CalaAccountSetId,

    pub individual_deposit_accounts_parent_account_set_id: CalaAccountSetId,
    pub government_entity_deposit_accounts_parent_account_set_id: CalaAccountSetId,
    pub private_company_deposit_accounts_parent_account_set_id: CalaAccountSetId,
    pub bank_deposit_accounts_parent_account_set_id: CalaAccountSetId,
    pub financial_institution_deposit_accounts_parent_account_set_id: CalaAccountSetId,
    pub non_domiciled_individual_deposit_accounts_parent_account_set_id: CalaAccountSetId,

    pub frozen_individual_deposit_accounts_parent_account_set_id: CalaAccountSetId,
    pub frozen_government_entity_deposit_accounts_parent_account_set_id: CalaAccountSetId,
    pub frozen_private_company_deposit_accounts_parent_account_set_id: CalaAccountSetId,
    pub frozen_bank_deposit_accounts_parent_account_set_id: CalaAccountSetId,
    pub frozen_financial_institution_deposit_accounts_parent_account_set_id: CalaAccountSetId,
    pub frozen_non_domiciled_individual_deposit_accounts_parent_account_set_id: CalaAccountSetId,
}
