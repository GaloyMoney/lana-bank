use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use audit::AuditInfo;

pub mod error;
mod templates;
mod velocity;

use cala_ledger::{
    account::*,
    account_set::{AccountSet, AccountSetMemberId, AccountSetUpdate, NewAccountSet},
    tx_template::Params,
    velocity::{NewVelocityControl, VelocityControlId},
    CalaLedger, Currency, DebitOrCredit, JournalId, LedgerOperation, TransactionId,
};

use crate::{
    chart_of_accounts_integration::ChartOfAccountsIntegrationConfig,
    primitives::{LedgerAccountId, LedgerAccountSetId, UsdCents},
    DepositAccountBalance, LedgerOmnibusAccountIds,
};

use error::*;

pub const DEPOSITS_ACCOUNT_SET_NAME: &str = "Deposits Account Set";
pub const DEPOSITS_ACCOUNT_SET_REF: &str = "deposits-account-set";

pub const DEPOSIT_OMNIBUS_ACCOUNT_SET_NAME: &str = "Deposit Omnibus Account Set";
pub const DEPOSIT_OMNIBUS_ACCOUNT_SET_REF: &str = "deposit-omnibus-account-set";
pub const DEPOSIT_OMNIBUS_ACCOUNT_REF: &str = "deposit-omnibus-account";

pub const DEPOSITS_VELOCITY_CONTROL_ID: uuid::Uuid =
    uuid::uuid!("00000000-0000-0000-0000-000000000001");

#[derive(Clone, Copy)]
pub struct InternalAccountSetDetails {
    id: LedgerAccountSetId,
    normal_balance_type: DebitOrCredit,
}

#[derive(Clone)]
pub struct DepositLedger {
    cala: CalaLedger,
    journal_id: JournalId,
    deposits_account_set: InternalAccountSetDetails,
    deposit_omnibus_account_ids: LedgerOmnibusAccountIds,
    usd: Currency,
    deposit_control_id: VelocityControlId,
}

impl DepositLedger {
    pub async fn init(
        cala: &CalaLedger,
        journal_id: JournalId,
    ) -> Result<Self, DepositLedgerError> {
        templates::RecordDeposit::init(cala).await?;
        templates::InitiateWithdraw::init(cala).await?;
        templates::CancelWithdraw::init(cala).await?;
        templates::ConfirmWithdraw::init(cala).await?;

        let deposits_normal_balance_type = DebitOrCredit::Credit;
        let deposits_account_set_id = Self::find_or_create_account_set(
            cala,
            journal_id,
            format!("{journal_id}:{DEPOSITS_ACCOUNT_SET_REF}"),
            DEPOSITS_ACCOUNT_SET_NAME.to_string(),
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
            cala: cala.clone(),
            journal_id,
            deposits_account_set: InternalAccountSetDetails {
                id: deposits_account_set_id,
                normal_balance_type: deposits_normal_balance_type,
            },
            deposit_omnibus_account_ids,
            deposit_control_id,
            usd: "USD".parse().expect("Could not parse 'USD'"),
        })
    }

    async fn find_or_create_account_set(
        cala: &CalaLedger,
        journal_id: JournalId,
        reference: String,
        name: String,
        normal_balance_type: DebitOrCredit,
    ) -> Result<LedgerAccountSetId, DepositLedgerError> {
        match cala
            .account_sets()
            .find_by_external_id(reference.to_string())
            .await
        {
            Ok(account_set) if account_set.values().journal_id != journal_id => {
                return Err(DepositLedgerError::JournalIdMismatch)
            }
            Ok(account_set) => return Ok(account_set.id),
            Err(e) if e.was_not_found() => (),
            Err(e) => return Err(e.into()),
        };

        let id = LedgerAccountSetId::new();
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
            .list_members(account_set_id, Default::default())
            .await?
            .entities;
        if !members.is_empty() {
            match members[0].id {
                AccountSetMemberId::Account(id) => {
                    return Ok(LedgerOmnibusAccountIds {
                        account_set_id,
                        account_id: id,
                    })
                }
                AccountSetMemberId::AccountSet(_) => {
                    return Err(DepositLedgerError::NonAccountMemberFoundInAccountSet(
                        account_set_id.to_string(),
                    ))
                }
            }
        }

        let mut op = cala.begin_operation().await?;
        let id = LedgerAccountId::new();
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
                op.rollback().await?;
                cala.accounts().find_by_external_id(reference).await?.id
            }
            Err(e) => return Err(e.into()),
        };

        Ok(LedgerOmnibusAccountIds {
            account_set_id,
            account_id,
        })
    }

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
            deposit_omnibus_account_id: self.deposit_omnibus_account_ids.account_id,
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
            deposit_omnibus_account_id: self.deposit_omnibus_account_ids.account_id,
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
            deposit_omnibus_account_id: self.deposit_omnibus_account_ids.account_id,
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
            deposit_omnibus_account_id: self.deposit_omnibus_account_ids.account_id,
        };

        self.cala
            .post_transaction_in_op(&mut op, tx_id, templates::CANCEL_WITHDRAW_CODE, params)
            .await?;
        op.commit().await?;
        Ok(())
    }

    pub async fn balance(
        &self,
        account_id: impl Into<AccountId>,
    ) -> Result<DepositAccountBalance, DepositLedgerError> {
        match self
            .cala
            .balances()
            .find(self.journal_id, account_id.into(), self.usd)
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

    pub async fn create_deposit_account(
        &self,
        op: es_entity::DbOp<'_>,
        id: impl Into<LedgerAccountId>,
        reference: String,
        name: String,
        description: String,
    ) -> Result<(), DepositLedgerError> {
        let id = id.into();

        let mut op = self.cala.ledger_operation_from_db_op(op);
        let new_ledger_account = NewAccount::builder()
            .id(id)
            .external_id(reference)
            .name(name)
            .description(description)
            .code(id.to_string())
            .normal_balance_type(self.deposits_account_set.normal_balance_type)
            .build()
            .expect("Could not build new account");
        let ledger_account = self
            .cala
            .accounts()
            .create_in_op(&mut op, new_ledger_account)
            .await?;
        self.cala
            .account_sets()
            .add_member_in_op(&mut op, self.deposits_account_set.id, ledger_account.id)
            .await?;

        self.add_deposit_control_to_account(&mut op, id).await?;

        op.commit().await?;

        Ok(())
    }

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

    pub async fn add_deposit_control_to_account(
        &self,
        op: &mut cala_ledger::LedgerOperation<'_>,
        account_id: impl Into<AccountId>,
    ) -> Result<(), DepositLedgerError> {
        self.cala
            .velocities()
            .attach_control_to_account_in_op(
                op,
                self.deposit_control_id,
                account_id.into(),
                Params::default(),
            )
            .await?;

        Ok(())
    }

    pub async fn get_chart_of_accounts_integration_config(
        &self,
    ) -> Result<Option<ChartOfAccountsIntegrationConfig>, DepositLedgerError> {
        let account_set = self
            .cala
            .account_sets()
            .find(self.deposits_account_set.id)
            .await?;
        if let Some(meta) = account_set.values().metadata.as_ref() {
            let meta: ChartOfAccountsIntegrationMeta =
                serde_json::from_value(meta.clone()).expect("Could not deserialize metadata");
            Ok(Some(meta.config))
        } else {
            Ok(None)
        }
    }

    async fn attach_charts_account_set<F>(
        &self,
        op: &mut LedgerOperation<'_>,
        account_sets: &mut HashMap<LedgerAccountSetId, AccountSet>,
        internal_account_set_id: LedgerAccountSetId,
        parent_account_set_id: LedgerAccountSetId,
        new_meta: &ChartOfAccountsIntegrationMeta,
        old_parent_id_getter: F,
    ) -> Result<(), DepositLedgerError>
    where
        F: FnOnce(ChartOfAccountsIntegrationMeta) -> LedgerAccountSetId,
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
        internal_account_set.update(update);
        self.cala
            .account_sets()
            .persist_in_op(op, &mut internal_account_set)
            .await?;

        Ok(())
    }

    pub async fn attach_chart_of_accounts_account_sets(
        &self,
        audit_info: AuditInfo,
        config: &ChartOfAccountsIntegrationConfig,
        deposit_accounts_parent_account_set_id: LedgerAccountSetId,
        omnibus_parent_account_set_id: LedgerAccountSetId,
    ) -> Result<(), DepositLedgerError> {
        let mut op = self.cala.begin_operation().await?;
        let mut account_sets = self
            .cala
            .account_sets()
            .find_all_in_op::<AccountSet>(
                &mut op,
                &[
                    self.deposits_account_set.id,
                    self.deposit_omnibus_account_ids.account_set_id,
                ],
            )
            .await?;

        let new_meta = ChartOfAccountsIntegrationMeta {
            config: config.clone(),
            deposit_accounts_parent_account_set_id,
            omnibus_parent_account_set_id,
            audit_info,
        };

        self.attach_charts_account_set(
            &mut op,
            &mut account_sets,
            self.deposits_account_set.id,
            deposit_accounts_parent_account_set_id,
            &new_meta,
            |meta| meta.deposit_accounts_parent_account_set_id,
        )
        .await?;
        self.attach_charts_account_set(
            &mut op,
            &mut account_sets,
            self.deposit_omnibus_account_ids.account_set_id,
            omnibus_parent_account_set_id,
            &new_meta,
            |meta| meta.omnibus_parent_account_set_id,
        )
        .await?;

        op.commit().await?;

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ChartOfAccountsIntegrationMeta {
    config: ChartOfAccountsIntegrationConfig,
    deposit_accounts_parent_account_set_id: LedgerAccountSetId,
    omnibus_parent_account_set_id: LedgerAccountSetId,
    audit_info: AuditInfo,
}
