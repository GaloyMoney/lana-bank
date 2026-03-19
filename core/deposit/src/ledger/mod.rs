use std::collections::HashMap;

use audit::SystemSubject;
use chrono::{DateTime, Utc};
use money::CurrencyMap;
use tracing::instrument;
use tracing_macros::record_error_severity;

use chart_primitives::EntityRef;
use es_entity::clock::ClockHandle;

mod deposit_accounts;
pub mod error;
mod templates;
mod velocity;

use cala_ledger::{
    CalaLedger, Currency, DebitOrCredit, JournalId, TransactionId,
    account::*,
    account_set::{AccountSetMemberId, NewAccountSet},
    tx_template::Params,
    velocity::{NewVelocityControl, VelocityControlId},
};

use crate::{
    DepositAccount, DepositAccountBalance, DepositReversalData, DepositSummaryAccountSetSpec,
    LedgerOmnibusAccountIds, WithdrawalReversalData,
    chart_of_accounts_integration::ResolvedChartOfAccountsIntegrationConfig,
    history::DepositAccountHistoryEntry,
    primitives::{
        CalaAccountId, CalaAccountSetId, CurrencyCode, DEPOSIT_ACCOUNT_ENTITY_TYPE,
        DEPOSIT_ACCOUNT_SET_CATALOG, DepositAccountType, DepositId, UsdCents, WithdrawalId,
    },
};

pub(super) use deposit_accounts::*;
use error::*;

pub const DEPOSITS_VELOCITY_CONTROL_ID: uuid::Uuid =
    uuid::uuid!("00000000-0000-0000-0000-000000000001");

#[derive(Clone, Copy, Debug)]
pub struct InternalAccountSetDetails {
    id: CalaAccountSetId,
    normal_balance_type: DebitOrCredit,
}

#[derive(Clone)]
pub struct DepositAccountSets {
    individual: CurrencyMap<InternalAccountSetDetails>,
    government_entity: CurrencyMap<InternalAccountSetDetails>,
    private_company: CurrencyMap<InternalAccountSetDetails>,
    bank: CurrencyMap<InternalAccountSetDetails>,
    financial_institution: CurrencyMap<InternalAccountSetDetails>,
    non_domiciled_company: CurrencyMap<InternalAccountSetDetails>,
}

impl DepositAccountSets {
    fn for_type(
        &self,
        deposit_account_type: DepositAccountType,
    ) -> &CurrencyMap<InternalAccountSetDetails> {
        match deposit_account_type {
            DepositAccountType::Individual => &self.individual,
            DepositAccountType::GovernmentEntity => &self.government_entity,
            DepositAccountType::PrivateCompany => &self.private_company,
            DepositAccountType::Bank => &self.bank,
            DepositAccountType::FinancialInstitution => &self.financial_institution,
            DepositAccountType::NonDomiciledCompany => &self.non_domiciled_company,
        }
    }

    fn find(
        &self,
        deposit_account_type: DepositAccountType,
        currency: CurrencyCode,
    ) -> Option<InternalAccountSetDetails> {
        self.for_type(deposit_account_type).get(&currency).copied()
    }
}

#[derive(Clone)]
pub struct DepositLedger {
    cala: CalaLedger,
    clock: ClockHandle,
    journal_id: JournalId,
    deposit_account_sets: DepositAccountSets,
    frozen_deposit_account_sets: DepositAccountSets,
    deposit_omnibus_account_ids: CurrencyMap<LedgerOmnibusAccountIds>,
    accounting_currency: CurrencyCode,
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

        let catalog = &DEPOSIT_ACCOUNT_SET_CATALOG;
        let deposit = catalog.deposit();
        let frozen = catalog.frozen();

        let mut deposit_ids: HashMap<&str, InternalAccountSetDetails> = HashMap::new();
        for spec in catalog.deposit_specs() {
            let id = Self::find_or_create_account_set(
                cala,
                journal_id,
                format!("{journal_id}:{}", spec.external_ref),
                spec.name.to_string(),
                spec.normal_balance_type,
            )
            .await?;
            deposit_ids.insert(
                spec.external_ref,
                InternalAccountSetDetails {
                    id,
                    normal_balance_type: spec.normal_balance_type,
                },
            );
        }

        let mut frozen_ids: HashMap<&str, InternalAccountSetDetails> = HashMap::new();
        for spec in catalog.frozen_specs() {
            let id = Self::find_or_create_account_set(
                cala,
                journal_id,
                format!("{journal_id}:{}", spec.external_ref),
                spec.name.to_string(),
                spec.normal_balance_type,
            )
            .await?;
            frozen_ids.insert(
                spec.external_ref,
                InternalAccountSetDetails {
                    id,
                    normal_balance_type: spec.normal_balance_type,
                },
            );
        }

        let mut omnibus_ids: HashMap<&str, LedgerOmnibusAccountIds> = HashMap::new();
        for spec in catalog.omnibus_specs() {
            let ids = Self::find_or_create_omnibus_account(
                cala,
                journal_id,
                format!("{journal_id}:{}", spec.account_set_ref),
                format!("{journal_id}:{}", spec.account_ref),
                spec.name.to_string(),
                spec.normal_balance_type,
                spec.currency,
            )
            .await?;
            omnibus_ids.insert(spec.account_set_ref, ids);
        }

        let deposit_omnibus_account_ids: CurrencyMap<_> = catalog
            .omnibus()
            .iter()
            .map(|(currency, spec)| (*currency, omnibus_ids[spec.account_set_ref].clone()))
            .collect();

        let deposit_control_id = Self::create_deposit_control(cala).await?;
        let overdraft_prevention_id = velocity::OverdraftPrevention::init(cala).await?;
        let currency_guard_id = velocity::CurrencyGuard::init(cala).await?;

        match cala
            .velocities()
            .add_limit_to_control(deposit_control_id, overdraft_prevention_id)
            .await
        {
            Ok(_)
            | Err(cala_ledger::velocity::error::VelocityError::LimitAlreadyAddedToControl) => {}
            Err(e) => return Err(e.into()),
        }
        match cala
            .velocities()
            .add_limit_to_control(deposit_control_id, currency_guard_id)
            .await
        {
            Ok(_)
            | Err(cala_ledger::velocity::error::VelocityError::LimitAlreadyAddedToControl) => {}
            Err(e) => return Err(e.into()),
        }
        for spec in catalog.deposit_specs() {
            Self::attach_single_currency_control_to_account_set(
                cala,
                deposit_control_id,
                deposit_ids[spec.external_ref].id,
                spec.currency,
            )
            .await?;
        }
        for spec in catalog.frozen_specs() {
            Self::attach_single_currency_control_to_account_set(
                cala,
                deposit_control_id,
                frozen_ids[spec.external_ref].id,
                spec.currency,
            )
            .await?;
        }
        for spec in catalog.omnibus_specs() {
            Self::attach_single_currency_control_to_account_set(
                cala,
                deposit_control_id,
                omnibus_ids[spec.account_set_ref].account_set_id,
                spec.currency,
            )
            .await?;
        }
        Ok(Self {
            clock,
            cala: cala.clone(),
            journal_id,
            deposit_account_sets: DepositAccountSets {
                individual: Self::internal_account_set_details(&deposit.individual, &deposit_ids),
                government_entity: Self::internal_account_set_details(
                    &deposit.government_entity,
                    &deposit_ids,
                ),
                private_company: Self::internal_account_set_details(
                    &deposit.private_company,
                    &deposit_ids,
                ),
                bank: Self::internal_account_set_details(&deposit.bank, &deposit_ids),
                financial_institution: Self::internal_account_set_details(
                    &deposit.financial_institution,
                    &deposit_ids,
                ),
                non_domiciled_company: Self::internal_account_set_details(
                    &deposit.non_domiciled_company,
                    &deposit_ids,
                ),
            },
            frozen_deposit_account_sets: DepositAccountSets {
                individual: Self::internal_account_set_details(&frozen.individual, &frozen_ids),
                government_entity: Self::internal_account_set_details(
                    &frozen.government_entity,
                    &frozen_ids,
                ),
                private_company: Self::internal_account_set_details(
                    &frozen.private_company,
                    &frozen_ids,
                ),
                bank: Self::internal_account_set_details(&frozen.bank, &frozen_ids),
                financial_institution: Self::internal_account_set_details(
                    &frozen.financial_institution,
                    &frozen_ids,
                ),
                non_domiciled_company: Self::internal_account_set_details(
                    &frozen.non_domiciled_company,
                    &frozen_ids,
                ),
            },
            deposit_omnibus_account_ids,
            accounting_currency: CurrencyCode::USD,
            deposit_control_id,
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
            Err(cala_ledger::account_set::error::AccountSetError::CouldNotFindByExternalId(_)) => {}
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
            Err(cala_ledger::account_set::error::AccountSetError::ExternalIdAlreadyExists(_)) => {
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
        currency: CurrencyCode,
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
                        currency,
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
            Err(cala_ledger::account::error::AccountError::ExternalIdAlreadyExists(_)) => {
                cala.accounts().find_by_external_id(reference).await?.id
            }
            Err(e) => return Err(e.into()),
        };

        Ok(LedgerOmnibusAccountIds {
            account_set_id,
            account_id,
            currency,
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
    #[instrument(name = "deposit_ledger.last_activity_date", skip_all, fields(account_id = tracing::field::Empty))]
    pub async fn last_activity_date(
        &self,
        id: impl Into<AccountId>,
    ) -> Result<Option<DateTime<Utc>>, DepositLedgerError> {
        let id = id.into();
        tracing::Span::current().record("account_id", tracing::field::debug(&id));

        let mut next = Some(es_entity::PaginatedQueryArgs::<
            cala_ledger::entry::EntriesByCreatedAtCursor,
        > {
            first: 10,
            after: None,
        });

        while let Some(query) = next.take() {
            let mut ret = self
                .cala
                .entries()
                .list_for_account_id(id, query, es_entity::ListDirection::Descending)
                .await?;

            for entry in ret.entities.drain(..) {
                if let Some(recorded_at) =
                    DepositAccountHistoryEntry::from(entry).activity_recorded_at()
                {
                    return Ok(Some(recorded_at));
                }
            }

            next = ret.into_next_query();
        }

        Ok(None)
    }

    #[record_error_severity]
    #[instrument(
        name = "deposit_ledger.record_deposit_in_op",
        skip_all,
        fields(entity_id = tracing::field::Empty, credit_account_id = tracing::field::Empty)
    )]
    pub async fn record_deposit_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        entity_id: DepositId,
        amount: UsdCents,
        credit_account_id: impl Into<AccountId>,
        initiated_by: &impl SystemSubject,
    ) -> Result<(), DepositLedgerError> {
        let tx_id = entity_id.into();
        tracing::Span::current().record("entity_id", tracing::field::debug(&entity_id));
        let credit_account_id = credit_account_id.into();
        tracing::Span::current().record(
            "credit_account_id",
            tracing::field::debug(&credit_account_id),
        );
        let deposit_omnibus_account_id = self
            .deposit_omnibus_account(self.accounting_currency)?
            .account_id;
        let params = templates::RecordDepositParams {
            entity_id: entity_id.into(),
            journal_id: self.journal_id,
            currency: self.accounting_currency,
            amount: amount.to_usd(),
            deposit_omnibus_account_id,
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
        name = "deposit_ledger.initiate_withdrawal_in_op",
        skip_all,
        fields(entity_id = tracing::field::Empty, credit_account_id = tracing::field::Empty)
    )]
    pub async fn initiate_withdrawal_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        entity_id: WithdrawalId,
        amount: UsdCents,
        credit_account_id: impl Into<AccountId>,
        initiated_by: &impl SystemSubject,
    ) -> Result<(), DepositLedgerError> {
        let tx_id = entity_id.into();
        tracing::Span::current().record("entity_id", tracing::field::debug(&entity_id));
        let credit_account_id = credit_account_id.into();
        tracing::Span::current().record(
            "credit_account_id",
            tracing::field::debug(&credit_account_id),
        );
        let deposit_omnibus_account_id = self
            .deposit_omnibus_account(self.accounting_currency)?
            .account_id;
        let params = templates::InitiateWithdrawParams {
            entity_id: entity_id.into(),
            journal_id: self.journal_id,
            deposit_omnibus_account_id,
            credit_account_id,
            amount: amount.to_usd(),
            currency: self.accounting_currency,
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
        name = "deposit_ledger.deny_withdrawal_in_op",
        skip_all,
        fields(entity_id = tracing::field::Empty, credit_account_id = tracing::field::Empty)
    )]
    pub async fn deny_withdrawal_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        entity_id: WithdrawalId,
        tx_id: impl Into<TransactionId>,
        amount: UsdCents,
        credit_account_id: impl Into<AccountId>,
        initiated_by: &impl SystemSubject,
    ) -> Result<(), DepositLedgerError> {
        let tx_id = tx_id.into();
        tracing::Span::current().record("entity_id", tracing::field::debug(&entity_id));
        tracing::Span::current().record("tx_id", tracing::field::debug(&tx_id));
        let credit_account_id = credit_account_id.into();
        tracing::Span::current().record(
            "credit_account_id",
            tracing::field::debug(&credit_account_id),
        );
        let deposit_omnibus_account_id = self
            .deposit_omnibus_account(self.accounting_currency)?
            .account_id;
        let params = templates::DenyWithdrawParams {
            entity_id: entity_id.into(),
            journal_id: self.journal_id,
            deposit_omnibus_account_id,
            credit_account_id,
            amount: amount.to_usd(),
            currency: self.accounting_currency,
            initiated_by,
            effective_date: self.clock.today(),
        };

        self.cala
            .post_transaction_in_op(op, tx_id, templates::DENY_WITHDRAW_CODE, params)
            .await?;

        Ok(())
    }

    #[record_error_severity]
    #[instrument(name = "deposit_ledger.revert_withdrawal_in_op", skip(self, op))]
    pub async fn revert_withdrawal_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        reversal_data: WithdrawalReversalData,
        initiated_by: &impl SystemSubject,
    ) -> Result<(), DepositLedgerError> {
        let deposit_omnibus_account_id = self
            .deposit_omnibus_account(self.accounting_currency)?
            .account_id;
        let params = templates::RevertWithdrawParams {
            entity_id: reversal_data.entity_id.into(),
            journal_id: self.journal_id,
            deposit_omnibus_account_id,
            credit_account_id: reversal_data.credit_account_id.into(),
            amount: reversal_data.amount.to_usd(),
            currency: self.accounting_currency,
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
    #[instrument(name = "deposit_ledger.revert_deposit_in_op", skip_all)]
    pub async fn revert_deposit_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        reversal_data: DepositReversalData,
        initiated_by: &impl SystemSubject,
    ) -> Result<(), DepositLedgerError> {
        let deposit_omnibus_account_id = self
            .deposit_omnibus_account(self.accounting_currency)?
            .account_id;
        let params = templates::RevertDepositParams {
            entity_id: reversal_data.entity_id.into(),
            journal_id: self.journal_id,
            deposit_omnibus_account_id,
            credit_account_id: reversal_data.credit_account_id.into(),
            correlation_id: reversal_data.correlation_id,
            external_id: reversal_data.external_id,
            amount: reversal_data.amount.to_usd(),
            currency: self.accounting_currency,
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
        initiated_by: &impl SystemSubject,
    ) -> Result<(), DepositLedgerError> {
        let balance = self.balance(account.id, account.currency).await?;

        if !balance.settled.is_zero() {
            let params = templates::FreezeAccountParams {
                journal_id: self.journal_id,
                account_id: account.account_ids.deposit_account_id,
                frozen_accounts_account_id: account.account_ids.frozen_deposit_account_id,
                amount: balance.settled.to_usd(),
                currency: self.accounting_currency,
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
        initiated_by: &impl SystemSubject,
    ) -> Result<(), DepositLedgerError> {
        let frozen_balance = self
            .balance(
                account.account_ids.frozen_deposit_account_id,
                account.currency,
            )
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
                currency: self.accounting_currency,
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
    #[instrument(name = "deposit_ledger.lock_account_in_op", skip(self, op))]
    pub async fn lock_account_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        account_id: AccountId,
    ) -> Result<(), DepositLedgerError> {
        self.cala.accounts().lock_in_op(op, account_id).await?;

        Ok(())
    }

    #[record_error_severity]
    #[instrument(
        name = "deposit_ledger.confirm_withdrawal_in_op",
        skip_all,
        fields(entity_id = tracing::field::Empty, tx_id = tracing::field::Empty, credit_account_id = tracing::field::Empty)
    )]
    pub async fn confirm_withdrawal_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        entity_id: WithdrawalId,
        tx_id: impl Into<TransactionId>,
        correlation_id: String,
        amount: UsdCents,
        credit_account_id: impl Into<AccountId>,
        external_id: String,
        initiated_by: &impl SystemSubject,
    ) -> Result<(), DepositLedgerError> {
        let tx_id = tx_id.into();
        tracing::Span::current().record("entity_id", tracing::field::debug(&entity_id));
        tracing::Span::current().record("tx_id", tracing::field::debug(&tx_id));
        let credit_account_id = credit_account_id.into();
        tracing::Span::current().record(
            "credit_account_id",
            tracing::field::debug(&credit_account_id),
        );
        let deposit_omnibus_account_id = self
            .deposit_omnibus_account(self.accounting_currency)?
            .account_id;
        let params = templates::ConfirmWithdrawParams {
            entity_id: entity_id.into(),
            journal_id: self.journal_id,
            currency: self.accounting_currency,
            amount: amount.to_usd(),
            deposit_omnibus_account_id,
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
        name = "deposit_ledger.cancel_withdrawal_in_op",
        skip_all,
        fields(entity_id = tracing::field::Empty, tx_id = tracing::field::Empty, credit_account_id = tracing::field::Empty)
    )]
    pub async fn cancel_withdrawal_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        entity_id: WithdrawalId,
        tx_id: impl Into<TransactionId>,
        amount: UsdCents,
        credit_account_id: impl Into<AccountId>,
        initiated_by: &impl SystemSubject,
    ) -> Result<(), DepositLedgerError> {
        let tx_id = tx_id.into();
        tracing::Span::current().record("entity_id", tracing::field::debug(&entity_id));
        tracing::Span::current().record("tx_id", tracing::field::debug(&tx_id));
        let credit_account_id = credit_account_id.into();
        tracing::Span::current().record(
            "credit_account_id",
            tracing::field::debug(&credit_account_id),
        );
        let deposit_omnibus_account_id = self
            .deposit_omnibus_account(self.accounting_currency)?
            .account_id;
        let params = templates::CancelWithdrawParams {
            entity_id: entity_id.into(),
            journal_id: self.journal_id,
            currency: self.accounting_currency,
            amount: amount.to_usd(),
            credit_account_id,
            deposit_omnibus_account_id,
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
        currency: CurrencyCode,
    ) -> Result<DepositAccountBalance, DepositLedgerError> {
        let account_id = account_id.into();
        tracing::Span::current().record("account_id", tracing::field::debug(&account_id));
        match self
            .cala
            .balances()
            .find(
                self.journal_id,
                account_id,
                self.cala_balance_currency(currency)?,
            )
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
    #[instrument(name = "deposit_ledger.create_deposit_accounts_in_op", skip_all)]
    pub async fn create_deposit_accounts_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        account: &DepositAccount,
        deposit_account_type: impl Into<DepositAccountType>,
    ) -> Result<(), DepositLedgerError> {
        let holder_id = account.account_holder_id;
        let deposit_account_type = deposit_account_type.into();
        let deposit_account_set =
            self.deposit_internal_account_set(deposit_account_type, account.currency)?;
        let frozen_account_set =
            self.frozen_deposit_internal_account_set(deposit_account_type, account.currency)?;

        let entity_ref = EntityRef::new(DEPOSIT_ACCOUNT_ENTITY_TYPE, account.id);
        let deposit_account_name = format!("Deposit Account {holder_id}");
        self.create_account_in_op(
            op,
            account.id,
            deposit_account_set,
            &format!("deposit-customer-account:{holder_id}"),
            &deposit_account_name,
            &deposit_account_name,
            entity_ref.clone(),
        )
        .await?;

        self.add_deposit_control_to_account_in_op(op, account.id, account.currency)
            .await?;

        let frozen_deposit_account_name = format!("Frozen Deposit Account {holder_id}");
        self.create_account_in_op(
            op,
            account.account_ids.frozen_deposit_account_id,
            frozen_account_set,
            &format!("frozen-deposit-customer-account:{holder_id}"),
            &frozen_deposit_account_name,
            &frozen_deposit_account_name,
            entity_ref,
        )
        .await?;

        Ok(())
    }

    fn deposit_internal_account_set(
        &self,
        deposit_account_type: DepositAccountType,
        currency: CurrencyCode,
    ) -> Result<InternalAccountSetDetails, DepositLedgerError> {
        self.deposit_account_sets
            .find(deposit_account_type, currency)
            .ok_or_else(|| DepositLedgerError::UnsupportedCurrencyForAccountType {
                account_type: deposit_account_type,
                currency,
            })
    }

    fn frozen_deposit_internal_account_set(
        &self,
        deposit_account_type: DepositAccountType,
        currency: CurrencyCode,
    ) -> Result<InternalAccountSetDetails, DepositLedgerError> {
        self.frozen_deposit_account_sets
            .find(deposit_account_type, currency)
            .ok_or_else(|| DepositLedgerError::UnsupportedCurrencyForAccountType {
                account_type: deposit_account_type,
                currency,
            })
    }

    fn internal_account_set_details(
        specs: &CurrencyMap<DepositSummaryAccountSetSpec>,
        ids: &HashMap<&str, InternalAccountSetDetails>,
    ) -> CurrencyMap<InternalAccountSetDetails> {
        specs
            .iter()
            .map(|(currency, spec)| (*currency, ids[spec.external_ref]))
            .collect()
    }

    fn deposit_omnibus_account(
        &self,
        currency: CurrencyCode,
    ) -> Result<&LedgerOmnibusAccountIds, DepositLedgerError> {
        self.deposit_omnibus_account_ids
            .get(&currency)
            .ok_or_else(|| DepositLedgerError::MissingOmnibusAccountForCurrency { currency })
    }

    // TODO: Remove this when Cala balance queries return currency (denomination of balance).
    fn cala_balance_currency(
        &self,
        currency: CurrencyCode,
    ) -> Result<Currency, DepositLedgerError> {
        match currency {
            CurrencyCode::USD => Ok(Currency::USD),
            CurrencyCode::BTC => Ok(Currency::BTC),
            currency => Err(DepositLedgerError::UnsupportedCalaCurrency { currency }),
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
        entity_ref: chart_primitives::EntityRef,
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
            .metadata(serde_json::json!({ "entity_ref": entity_ref }))
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
            Err(cala_ledger::velocity::error::VelocityError::ControlIdAlreadyExists(_)) => {
                Ok(DEPOSITS_VELOCITY_CONTROL_ID.into())
            }
            Err(e) => Err(e.into()),
            Ok(control) => Ok(control.id()),
        }
    }

    #[record_error_severity]
    #[instrument(
        name = "deposit_ledger.add_deposit_control_to_account_in_op",
        skip_all,
        fields(account_id = tracing::field::Empty)
    )]
    pub async fn add_deposit_control_to_account_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        account_id: impl Into<AccountId>,
        currency: CurrencyCode,
    ) -> Result<(), DepositLedgerError> {
        let account_id = account_id.into();
        tracing::Span::current().record("account_id", tracing::field::debug(&account_id));
        let mut params = Params::new();
        params.insert("account_currency", currency.to_string());
        self.cala
            .velocities()
            .attach_control_to_account_in_op(op, self.deposit_control_id, account_id, params)
            .await?;

        Ok(())
    }

    #[record_error_severity]
    #[instrument(
        name = "deposit_ledger.attach_single_currency_control_to_account_set",
        skip(cala),
        fields(account_set_id = tracing::field::Empty, currency = %currency)
    )]
    async fn attach_single_currency_control_to_account_set(
        cala: &CalaLedger,
        control_id: VelocityControlId,
        account_set_id: CalaAccountSetId,
        currency: CurrencyCode,
    ) -> Result<(), DepositLedgerError> {
        tracing::Span::current().record("account_set_id", tracing::field::debug(&account_set_id));

        let mut params = Params::new();
        params.insert("account_currency", currency.to_string());

        match cala
            .velocities()
            .attach_control_to_account_set(control_id, account_set_id.into(), params)
            .await
        {
            Ok(_) => Ok(()),
            Err(cala_ledger::velocity::error::VelocityError::Sqlx(sqlx::Error::Database(
                db_err,
            ))) if db_err.constraint().is_some_and(|constraint| {
                constraint
                    .contains("cala_velocity_account_controls_account_id_velocity_control_id_key")
            }) =>
            {
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }

    #[record_error_severity]
    #[instrument(name = "deposit_ledger.attach_charts_account_set_in_op", skip_all)]
    async fn attach_charts_account_set_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        internal_account_set_id: CalaAccountSetId,
        new_parent_account_set_id: CalaAccountSetId,
        old_parent_account_set_id: Option<CalaAccountSetId>,
    ) -> Result<(), DepositLedgerError> {
        if let Some(old_parent_account_set_id) = old_parent_account_set_id {
            self.cala
                .account_sets()
                .remove_member_in_op(op, old_parent_account_set_id, internal_account_set_id)
                .await?;
        }

        self.cala
            .account_sets()
            .add_member_in_op(op, new_parent_account_set_id, internal_account_set_id)
            .await?;

        Ok(())
    }

    #[record_error_severity]
    #[instrument(
        name = "deposit_ledger.attach_chart_of_accounts_account_sets_in_op",
        skip_all
    )]
    pub(crate) async fn attach_chart_of_accounts_account_sets_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        new_integration_config: &ResolvedChartOfAccountsIntegrationConfig,
        old_integration_config: Option<&ResolvedChartOfAccountsIntegrationConfig>,
    ) -> Result<(), DepositLedgerError> {
        let ResolvedChartOfAccountsIntegrationConfig {
            config: _,

            omnibus_parent_account_set_id,
            individual_deposit_accounts_parent_account_set_id,
            government_entity_deposit_accounts_parent_account_set_id,
            private_company_deposit_accounts_parent_account_set_id,
            bank_deposit_accounts_parent_account_set_id,
            financial_institution_deposit_accounts_parent_account_set_id,
            non_domiciled_company_deposit_accounts_parent_account_set_id,
            frozen_individual_deposit_accounts_parent_account_set_id,
            frozen_government_entity_deposit_accounts_parent_account_set_id,
            frozen_private_company_deposit_accounts_parent_account_set_id,
            frozen_bank_deposit_accounts_parent_account_set_id,
            frozen_financial_institution_deposit_accounts_parent_account_set_id,
            frozen_non_domiciled_company_deposit_accounts_parent_account_set_id,
        } = &new_integration_config;

        self.attach_charts_account_set_in_op(
            op,
            self.deposit_omnibus_account(self.accounting_currency)?
                .account_set_id,
            *omnibus_parent_account_set_id,
            old_integration_config.map(|config| config.omnibus_parent_account_set_id),
        )
        .await?;

        self.attach_charts_account_set_in_op(
            op,
            self.deposit_internal_account_set(DepositAccountType::Individual, CurrencyCode::USD)?
                .id,
            *individual_deposit_accounts_parent_account_set_id,
            old_integration_config
                .map(|config| config.individual_deposit_accounts_parent_account_set_id),
        )
        .await?;

        self.attach_charts_account_set_in_op(
            op,
            self.deposit_internal_account_set(
                DepositAccountType::GovernmentEntity,
                CurrencyCode::USD,
            )?
            .id,
            *government_entity_deposit_accounts_parent_account_set_id,
            old_integration_config
                .map(|config| config.government_entity_deposit_accounts_parent_account_set_id),
        )
        .await?;

        self.attach_charts_account_set_in_op(
            op,
            self.deposit_internal_account_set(
                DepositAccountType::PrivateCompany,
                CurrencyCode::USD,
            )?
            .id,
            *private_company_deposit_accounts_parent_account_set_id,
            old_integration_config
                .map(|config| config.private_company_deposit_accounts_parent_account_set_id),
        )
        .await?;

        self.attach_charts_account_set_in_op(
            op,
            self.deposit_internal_account_set(DepositAccountType::Bank, CurrencyCode::USD)?
                .id,
            *bank_deposit_accounts_parent_account_set_id,
            old_integration_config.map(|config| config.bank_deposit_accounts_parent_account_set_id),
        )
        .await?;

        self.attach_charts_account_set_in_op(
            op,
            self.deposit_internal_account_set(
                DepositAccountType::FinancialInstitution,
                CurrencyCode::USD,
            )?
            .id,
            *financial_institution_deposit_accounts_parent_account_set_id,
            old_integration_config
                .map(|config| config.financial_institution_deposit_accounts_parent_account_set_id),
        )
        .await?;

        self.attach_charts_account_set_in_op(
            op,
            self.deposit_internal_account_set(
                DepositAccountType::NonDomiciledCompany,
                CurrencyCode::USD,
            )?
            .id,
            *non_domiciled_company_deposit_accounts_parent_account_set_id,
            old_integration_config
                .map(|config| config.non_domiciled_company_deposit_accounts_parent_account_set_id),
        )
        .await?;

        self.attach_charts_account_set_in_op(
            op,
            self.frozen_deposit_internal_account_set(
                DepositAccountType::Individual,
                CurrencyCode::USD,
            )?
            .id,
            *frozen_individual_deposit_accounts_parent_account_set_id,
            old_integration_config
                .map(|config| config.frozen_individual_deposit_accounts_parent_account_set_id),
        )
        .await?;

        self.attach_charts_account_set_in_op(
            op,
            self.frozen_deposit_internal_account_set(
                DepositAccountType::GovernmentEntity,
                CurrencyCode::USD,
            )?
            .id,
            *frozen_government_entity_deposit_accounts_parent_account_set_id,
            old_integration_config.map(|config| {
                config.frozen_government_entity_deposit_accounts_parent_account_set_id
            }),
        )
        .await?;

        self.attach_charts_account_set_in_op(
            op,
            self.frozen_deposit_internal_account_set(
                DepositAccountType::PrivateCompany,
                CurrencyCode::USD,
            )?
            .id,
            *frozen_private_company_deposit_accounts_parent_account_set_id,
            old_integration_config
                .map(|config| config.frozen_private_company_deposit_accounts_parent_account_set_id),
        )
        .await?;

        self.attach_charts_account_set_in_op(
            op,
            self.frozen_deposit_internal_account_set(DepositAccountType::Bank, CurrencyCode::USD)?
                .id,
            *frozen_bank_deposit_accounts_parent_account_set_id,
            old_integration_config
                .map(|config| config.frozen_bank_deposit_accounts_parent_account_set_id),
        )
        .await?;

        self.attach_charts_account_set_in_op(
            op,
            self.frozen_deposit_internal_account_set(
                DepositAccountType::FinancialInstitution,
                CurrencyCode::USD,
            )?
            .id,
            *frozen_financial_institution_deposit_accounts_parent_account_set_id,
            old_integration_config.map(|config| {
                config.frozen_financial_institution_deposit_accounts_parent_account_set_id
            }),
        )
        .await?;

        self.attach_charts_account_set_in_op(
            op,
            self.frozen_deposit_internal_account_set(
                DepositAccountType::NonDomiciledCompany,
                CurrencyCode::USD,
            )?
            .id,
            *frozen_non_domiciled_company_deposit_accounts_parent_account_set_id,
            old_integration_config.map(|config| {
                config.frozen_non_domiciled_company_deposit_accounts_parent_account_set_id
            }),
        )
        .await?;

        Ok(())
    }
}
