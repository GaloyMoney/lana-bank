use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use audit::AuditInfo;

mod constants;
mod credit_facility_accounts;
pub mod error;
mod templates;
mod velocity;

use cala_ledger::{
    account::NewAccount,
    account_set::{AccountSet, AccountSetMemberId, AccountSetUpdate, NewAccountSet},
    velocity::{NewVelocityControl, VelocityControlId},
    CalaLedger, Currency, DebitOrCredit, JournalId, LedgerOperation, TransactionId,
};

use crate::{
    primitives::{
        CollateralAction, CreditFacilityId, LedgerAccountId, LedgerAccountSetId,
        LedgerOmnibusAccountIds, Satoshis, UsdCents,
    },
    ChartOfAccountsIntegrationConfig, DisbursedReceivableAccountType,
};

use constants::*;
pub use credit_facility_accounts::*;
use error::*;

#[derive(Debug, Clone)]
pub struct CreditFacilityCollateralUpdate {
    pub tx_id: TransactionId,
    pub abs_diff: Satoshis,
    pub action: CollateralAction,
    pub credit_facility_account_ids: CreditFacilityAccountIds,
}

#[derive(Clone, Copy)]
pub struct InternalAccountSetDetails {
    id: LedgerAccountSetId,
    normal_balance_type: DebitOrCredit,
}

#[derive(Clone, Copy)]
pub struct DisbursedReceivableAccountSets {
    individual: InternalAccountSetDetails,
    government_entity: InternalAccountSetDetails,
    private_company: InternalAccountSetDetails,
    bank: InternalAccountSetDetails,
    financial_institution: InternalAccountSetDetails,
    foreign_agency_or_subsidiary: InternalAccountSetDetails,
    non_domiciled_company: InternalAccountSetDetails,
}

impl DisbursedReceivableAccountSets {
    fn account_set_ids(&self) -> Vec<LedgerAccountSetId> {
        vec![
            self.individual.id,
            self.government_entity.id,
            self.private_company.id,
            self.bank.id,
            self.financial_institution.id,
            self.foreign_agency_or_subsidiary.id,
            self.non_domiciled_company.id,
        ]
    }
}

#[derive(Clone, Copy)]
pub struct DisbursedReceivable {
    short_term: DisbursedReceivableAccountSets,
    long_term: DisbursedReceivableAccountSets,
}

#[derive(Clone, Copy)]
pub struct CreditFacilityInternalAccountSets {
    pub facility: InternalAccountSetDetails,
    pub collateral: InternalAccountSetDetails,
    pub disbursed_receivable: DisbursedReceivable,
    pub interest_receivable: InternalAccountSetDetails,
    pub interest_income: InternalAccountSetDetails,
    pub fee_income: InternalAccountSetDetails,
}

impl CreditFacilityInternalAccountSets {
    fn account_set_ids(&self) -> Vec<LedgerAccountSetId> {
        let mut ids = vec![
            self.facility.id,
            self.collateral.id,
            self.interest_receivable.id,
            self.interest_income.id,
            self.fee_income.id,
        ];
        ids.extend(
            self.disbursed_receivable
                .short_term
                .account_set_ids()
                .into_iter()
                .chain(self.disbursed_receivable.long_term.account_set_ids()),
        );

        ids
    }
}

#[derive(Clone)]
pub struct CreditLedger {
    cala: CalaLedger,
    journal_id: JournalId,
    facility_omnibus_account_ids: LedgerOmnibusAccountIds,
    collateral_omnibus_account_ids: LedgerOmnibusAccountIds,
    internal_account_sets: CreditFacilityInternalAccountSets,
    credit_facility_control_id: VelocityControlId,
    usd: Currency,
    btc: Currency,
}

impl CreditLedger {
    pub async fn init(cala: &CalaLedger, journal_id: JournalId) -> Result<Self, CreditLedgerError> {
        templates::AddCollateral::init(cala).await?;
        templates::ActivateCreditFacility::init(cala).await?;
        templates::RemoveCollateral::init(cala).await?;
        templates::RecordPayment::init(cala).await?;
        templates::CreditFacilityIncurInterest::init(cala).await?;
        templates::CreditFacilityAccrueInterest::init(cala).await?;
        templates::InitiateDisbursal::init(cala).await?;
        templates::CancelDisbursal::init(cala).await?;
        templates::SettleDisbursal::init(cala).await?;

        let collateral_omnibus_normal_balance_type = DebitOrCredit::Debit;
        let collateral_omnibus_account_ids = Self::find_or_create_omnibus_account(
            cala,
            journal_id,
            format!("{journal_id}:{CREDIT_COLLATERAL_OMNIBUS_ACCOUNT_SET_REF}"),
            format!("{journal_id}:{CREDIT_COLLATERAL_OMNIBUS_ACCOUNT_REF}"),
            CREDIT_COLLATERAL_OMNIBUS_ACCOUNT_SET_NAME.to_string(),
            collateral_omnibus_normal_balance_type,
        )
        .await?;

        let facility_omnibus_normal_balance_type = DebitOrCredit::Debit;
        let facility_omnibus_account_ids = Self::find_or_create_omnibus_account(
            cala,
            journal_id,
            format!("{journal_id}:{CREDIT_FACILITY_OMNIBUS_ACCOUNT_SET_REF}"),
            format!("{journal_id}:{CREDIT_FACILITY_OMNIBUS_ACCOUNT_REF}"),
            CREDIT_FACILITY_OMNIBUS_ACCOUNT_SET_NAME.to_string(),
            facility_omnibus_normal_balance_type,
        )
        .await?;

        let facility_normal_balance_type = DebitOrCredit::Credit;
        let facility_account_set_id = Self::find_or_create_account_set(
            cala,
            journal_id,
            format!("{journal_id}:{CREDIT_FACILITY_REMAINING_ACCOUNT_SET_REF}"),
            CREDIT_FACILITY_REMAINING_ACCOUNT_SET_NAME.to_string(),
            facility_normal_balance_type,
        )
        .await?;

        let collateral_normal_balance_type = DebitOrCredit::Credit;
        let collateral_account_set_id = Self::find_or_create_account_set(
            cala,
            journal_id,
            format!("{journal_id}:{CREDIT_COLLATERAL_ACCOUNT_SET_REF}"),
            CREDIT_COLLATERAL_ACCOUNT_SET_NAME.to_string(),
            collateral_normal_balance_type,
        )
        .await?;

        let disbursed_receivable_normal_balance_type = DebitOrCredit::Debit;
        let short_term_individual_disbursed_receivable_account_set_id =
            Self::find_or_create_account_set(
                cala,
                journal_id,
                format!(
                "{journal_id}:{SHORT_TERM_CREDIT_INDIVIDUAL_DISBURSED_RECEIVABLE_ACCOUNT_SET_REF}"
            ),
                SHORT_TERM_CREDIT_INDIVIDUAL_DISBURSED_RECEIVABLE_ACCOUNT_SET_NAME.to_string(),
                disbursed_receivable_normal_balance_type,
            )
            .await?;
        let short_term_government_entity_disbursed_receivable_account_set_id =
            Self::find_or_create_account_set(
                cala,
                journal_id,
                format!(
                    "{journal_id}:{SHORT_TERM_CREDIT_GOVERNMENT_ENTITY_DISBURSED_RECEIVABLE_ACCOUNT_SET_REF}"
                ),
                SHORT_TERM_CREDIT_GOVERNMENT_ENTITY_DISBURSED_RECEIVABLE_ACCOUNT_SET_NAME
                    .to_string(),
                disbursed_receivable_normal_balance_type,
            )
            .await?;
        let short_term_private_company_disbursed_receivable_account_set_id = Self::find_or_create_account_set(
            cala,
            journal_id,
            format!("{journal_id}:{SHORT_TERM_CREDIT_PRIVATE_COMPANY_DISBURSED_RECEIVABLE_ACCOUNT_SET_REF}"),
            SHORT_TERM_CREDIT_PRIVATE_COMPANY_DISBURSED_RECEIVABLE_ACCOUNT_SET_NAME.to_string(),
            disbursed_receivable_normal_balance_type,
        )
        .await?;
        let short_term_bank_disbursed_receivable_account_set_id = Self::find_or_create_account_set(
            cala,
            journal_id,
            format!("{journal_id}:{SHORT_TERM_CREDIT_BANK_DISBURSED_RECEIVABLE_ACCOUNT_SET_REF}"),
            SHORT_TERM_CREDIT_BANK_DISBURSED_RECEIVABLE_ACCOUNT_SET_NAME.to_string(),
            disbursed_receivable_normal_balance_type,
        )
        .await?;
        let short_term_financial_institution_disbursed_receivable_account_set_id =
            Self::find_or_create_account_set(
                cala,
                journal_id,
                format!("{journal_id}:{SHORT_TERM_CREDIT_FINANCIAL_INSTITUTION_DISBURSED_RECEIVABLE_ACCOUNT_SET_REF}"),
                SHORT_TERM_CREDIT_FINANCIAL_INSTITUTION_DISBURSED_RECEIVABLE_ACCOUNT_SET_NAME.to_string(),
                disbursed_receivable_normal_balance_type,
            )
            .await?;
        let short_term_foreign_agency_or_subsidiary_disbursed_receivable_account_set_id =
            Self::find_or_create_account_set(
                cala,
                journal_id,
                format!("{journal_id}:{SHORT_TERM_CREDIT_FOREIGN_AGENCY_OR_SUBSIDIARY_DISBURSED_RECEIVABLE_ACCOUNT_SET_REF}"),
                SHORT_TERM_CREDIT_FOREIGN_AGENCY_OR_SUBSIDIARY_DISBURSED_RECEIVABLE_ACCOUNT_SET_NAME
                    .to_string(),
                disbursed_receivable_normal_balance_type,
            )
            .await?;
        let short_term_non_domiciled_company_disbursed_receivable_account_set_id =
            Self::find_or_create_account_set(
                cala,
                journal_id,
                format!("{journal_id}:{SHORT_TERM_CREDIT_NON_DOMICILED_COMPANY_DISBURSED_RECEIVABLE_ACCOUNT_SET_REF}"),
                SHORT_TERM_CREDIT_NON_DOMICILED_COMPANY_DISBURSED_RECEIVABLE_ACCOUNT_SET_NAME.to_string(),
                disbursed_receivable_normal_balance_type,
            )
            .await?;

        let long_term_individual_disbursed_receivable_account_set_id = Self::find_or_create_account_set(
            cala,
            journal_id,
            format!("{journal_id}:{LONG_TERM_CREDIT_INDIVIDUAL_DISBURSED_RECEIVABLE_ACCOUNT_SET_REF}"),
            LONG_TERM_CREDIT_INDIVIDUAL_DISBURSED_RECEIVABLE_ACCOUNT_SET_NAME.to_string(),
            disbursed_receivable_normal_balance_type,
        )
        .await?;
        let long_term_government_entity_disbursed_receivable_account_set_id = Self::find_or_create_account_set(
            cala,
            journal_id,
            format!("{journal_id}:{LONG_TERM_CREDIT_GOVERNMENT_ENTITY_DISBURSED_RECEIVABLE_ACCOUNT_SET_REF}"),
            LONG_TERM_CREDIT_GOVERNMENT_ENTITY_DISBURSED_RECEIVABLE_ACCOUNT_SET_NAME.to_string(),
            disbursed_receivable_normal_balance_type,
        )
        .await?;
        let long_term_private_company_disbursed_receivable_account_set_id = Self::find_or_create_account_set(
            cala,
            journal_id,
            format!("{journal_id}:{LONG_TERM_CREDIT_PRIVATE_COMPANY_DISBURSED_RECEIVABLE_ACCOUNT_SET_REF}"),
            LONG_TERM_CREDIT_PRIVATE_COMPANY_DISBURSED_RECEIVABLE_ACCOUNT_SET_NAME.to_string(),
            disbursed_receivable_normal_balance_type,
        )
        .await?;
        let long_term_bank_disbursed_receivable_account_set_id = Self::find_or_create_account_set(
            cala,
            journal_id,
            format!("{journal_id}:{LONG_TERM_CREDIT_BANK_DISBURSED_RECEIVABLE_ACCOUNT_SET_REF}"),
            LONG_TERM_CREDIT_BANK_DISBURSED_RECEIVABLE_ACCOUNT_SET_NAME.to_string(),
            disbursed_receivable_normal_balance_type,
        )
        .await?;
        let long_term_financial_institution_disbursed_receivable_account_set_id = Self::find_or_create_account_set(
            cala,
            journal_id,
            format!("{journal_id}:{LONG_TERM_CREDIT_FINANCIAL_INSTITUTION_DISBURSED_RECEIVABLE_ACCOUNT_SET_REF}"),
            LONG_TERM_CREDIT_FINANCIAL_INSTITUTION_DISBURSED_RECEIVABLE_ACCOUNT_SET_NAME.to_string(),
            disbursed_receivable_normal_balance_type,
        )
        .await?;
        let long_term_foreign_agency_or_subsidiary_disbursed_receivable_account_set_id = Self::find_or_create_account_set(
            cala,
            journal_id,
            format!("{journal_id}:{LONG_TERM_CREDIT_FOREIGN_AGENCY_OR_SUBSIDIARY_DISBURSED_RECEIVABLE_ACCOUNT_SET_REF}"),
            LONG_TERM_CREDIT_FOREIGN_AGENCY_OR_SUBSIDIARY_DISBURSED_RECEIVABLE_ACCOUNT_SET_NAME.to_string(),
            disbursed_receivable_normal_balance_type,
        )
        .await?;
        let long_term_non_domiciled_company_disbursed_receivable_account_set_id = Self::find_or_create_account_set(
            cala,
            journal_id,
            format!("{journal_id}:{LONG_TERM_CREDIT_NON_DOMICILED_COMPANY_DISBURSED_RECEIVABLE_ACCOUNT_SET_REF}"),
            LONG_TERM_CREDIT_NON_DOMICILED_COMPANY_DISBURSED_RECEIVABLE_ACCOUNT_SET_NAME.to_string(),
            disbursed_receivable_normal_balance_type,
        )
        .await?;

        let interest_receivable_normal_balance_type = DebitOrCredit::Debit;
        let interest_receivable_account_set_id = Self::find_or_create_account_set(
            cala,
            journal_id,
            format!("{journal_id}:{CREDIT_INTEREST_RECEIVABLE_ACCOUNT_SET_REF}"),
            CREDIT_INTEREST_RECEIVABLE_ACCOUNT_SET_NAME.to_string(),
            interest_receivable_normal_balance_type,
        )
        .await?;

        let interest_income_normal_balance_type = DebitOrCredit::Credit;
        let interest_income_account_set_id = Self::find_or_create_account_set(
            cala,
            journal_id,
            format!("{journal_id}:{CREDIT_INTEREST_INCOME_ACCOUNT_SET_REF}"),
            CREDIT_INTEREST_INCOME_ACCOUNT_SET_NAME.to_string(),
            interest_income_normal_balance_type,
        )
        .await?;

        let fee_income_normal_balance_type = DebitOrCredit::Credit;
        let fee_income_account_set_id = Self::find_or_create_account_set(
            cala,
            journal_id,
            format!("{journal_id}:{CREDIT_FEE_INCOME_ACCOUNT_SET_REF}"),
            CREDIT_FEE_INCOME_ACCOUNT_SET_NAME.to_string(),
            fee_income_normal_balance_type,
        )
        .await?;

        let disbursed_receivable = DisbursedReceivable {
            short_term: DisbursedReceivableAccountSets {
                individual: InternalAccountSetDetails {
                    id: short_term_individual_disbursed_receivable_account_set_id,
                    normal_balance_type: disbursed_receivable_normal_balance_type,
                },
                government_entity: InternalAccountSetDetails {
                    id: short_term_government_entity_disbursed_receivable_account_set_id,
                    normal_balance_type: disbursed_receivable_normal_balance_type,
                },
                private_company: InternalAccountSetDetails {
                    id: short_term_private_company_disbursed_receivable_account_set_id,
                    normal_balance_type: disbursed_receivable_normal_balance_type,
                },
                bank: InternalAccountSetDetails {
                    id: short_term_bank_disbursed_receivable_account_set_id,
                    normal_balance_type: disbursed_receivable_normal_balance_type,
                },
                financial_institution: InternalAccountSetDetails {
                    id: short_term_financial_institution_disbursed_receivable_account_set_id,
                    normal_balance_type: disbursed_receivable_normal_balance_type,
                },
                foreign_agency_or_subsidiary: InternalAccountSetDetails {
                    id: short_term_foreign_agency_or_subsidiary_disbursed_receivable_account_set_id,
                    normal_balance_type: disbursed_receivable_normal_balance_type,
                },
                non_domiciled_company: InternalAccountSetDetails {
                    id: short_term_non_domiciled_company_disbursed_receivable_account_set_id,
                    normal_balance_type: disbursed_receivable_normal_balance_type,
                },
            },
            long_term: DisbursedReceivableAccountSets {
                individual: InternalAccountSetDetails {
                    id: long_term_individual_disbursed_receivable_account_set_id,
                    normal_balance_type: disbursed_receivable_normal_balance_type,
                },
                government_entity: InternalAccountSetDetails {
                    id: long_term_government_entity_disbursed_receivable_account_set_id,
                    normal_balance_type: disbursed_receivable_normal_balance_type,
                },
                private_company: InternalAccountSetDetails {
                    id: long_term_private_company_disbursed_receivable_account_set_id,
                    normal_balance_type: disbursed_receivable_normal_balance_type,
                },
                bank: InternalAccountSetDetails {
                    id: long_term_bank_disbursed_receivable_account_set_id,
                    normal_balance_type: disbursed_receivable_normal_balance_type,
                },
                financial_institution: InternalAccountSetDetails {
                    id: long_term_financial_institution_disbursed_receivable_account_set_id,
                    normal_balance_type: disbursed_receivable_normal_balance_type,
                },
                foreign_agency_or_subsidiary: InternalAccountSetDetails {
                    id: long_term_foreign_agency_or_subsidiary_disbursed_receivable_account_set_id,
                    normal_balance_type: disbursed_receivable_normal_balance_type,
                },
                non_domiciled_company: InternalAccountSetDetails {
                    id: long_term_non_domiciled_company_disbursed_receivable_account_set_id,
                    normal_balance_type: disbursed_receivable_normal_balance_type,
                },
            },
        };

        let internal_account_sets = CreditFacilityInternalAccountSets {
            facility: InternalAccountSetDetails {
                id: facility_account_set_id,
                normal_balance_type: facility_normal_balance_type,
            },
            collateral: InternalAccountSetDetails {
                id: collateral_account_set_id,
                normal_balance_type: collateral_normal_balance_type,
            },
            disbursed_receivable,
            interest_receivable: InternalAccountSetDetails {
                id: interest_receivable_account_set_id,
                normal_balance_type: interest_receivable_normal_balance_type,
            },
            interest_income: InternalAccountSetDetails {
                id: interest_income_account_set_id,
                normal_balance_type: interest_income_normal_balance_type,
            },
            fee_income: InternalAccountSetDetails {
                id: fee_income_account_set_id,
                normal_balance_type: fee_income_normal_balance_type,
            },
        };

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
            facility_omnibus_account_ids,
            collateral_omnibus_account_ids,
            internal_account_sets,
            credit_facility_control_id,
            usd: "USD".parse().expect("Could not parse 'USD'"),
            btc: "BTC".parse().expect("Could not parse 'BTC'"),
        })
    }

    async fn find_or_create_account_set(
        cala: &CalaLedger,
        journal_id: JournalId,
        reference: String,
        name: String,
        normal_balance_type: DebitOrCredit,
    ) -> Result<LedgerAccountSetId, CreditLedgerError> {
        match cala
            .account_sets()
            .find_by_external_id(reference.to_string())
            .await
        {
            Ok(account_set) if account_set.values().journal_id != journal_id => {
                return Err(CreditLedgerError::JournalIdMismatch)
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
    ) -> Result<LedgerOmnibusAccountIds, CreditLedgerError> {
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
                    return Err(CreditLedgerError::NonAccountMemberFoundInAccountSet(
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
                op.commit().await?;
                cala.accounts().find_by_external_id(reference).await?.id
            }
            Err(e) => return Err(e.into()),
        };

        Ok(LedgerOmnibusAccountIds {
            account_set_id,
            account_id,
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
                            bank_collateral_account_id: self
                                .collateral_omnibus_account_ids
                                .account_id,
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
                            bank_collateral_account_id: self
                                .collateral_omnibus_account_ids
                                .account_id,
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
        debit_account_id: LedgerAccountId,
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
                    bank_collateral_account_id: self.collateral_omnibus_account_ids.account_id,
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
                templates::ACTIVATE_CREDIT_FACILITY_CODE,
                templates::ActivateCreditFacilityParams {
                    journal_id: self.journal_id,
                    credit_omnibus_account: self.facility_omnibus_account_ids.account_id,
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
                    credit_omnibus_account: self.facility_omnibus_account_ids.account_id,
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
                        credit_omnibus_account: self.facility_omnibus_account_ids.account_id,
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
                        credit_omnibus_account: self.facility_omnibus_account_ids.account_id,
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
        account_id: impl Into<LedgerAccountId>,
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

    async fn create_account_in_op(
        &self,
        op: &mut LedgerOperation<'_>,
        id: impl Into<LedgerAccountId>,
        parent_account_set: InternalAccountSetDetails,
        reference: &str,
        name: &str,
        description: &str,
    ) -> Result<(), CreditLedgerError> {
        let id = id.into();

        let new_ledger_account = NewAccount::builder()
            .id(id)
            .external_id(reference)
            .name(name)
            .description(description)
            .code(id.to_string())
            .normal_balance_type(parent_account_set.normal_balance_type)
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

    fn disbursed_internal_account_set_from_type(
        &self,
        disbursed_account_type: DisbursedReceivableAccountType,
        is_short_term: bool,
    ) -> InternalAccountSetDetails {
        let term_type = if is_short_term {
            &self.internal_account_sets.disbursed_receivable.short_term
        } else {
            &self.internal_account_sets.disbursed_receivable.long_term
        };

        match disbursed_account_type {
            DisbursedReceivableAccountType::Individual => term_type.individual,
            DisbursedReceivableAccountType::GovernmentEntity => term_type.government_entity,
            DisbursedReceivableAccountType::PrivateCompany => term_type.private_company,
            DisbursedReceivableAccountType::Bank => term_type.bank,
            DisbursedReceivableAccountType::FinancialInstitution => term_type.financial_institution,
            DisbursedReceivableAccountType::ForeignAgencyOrSubsidiary => {
                term_type.foreign_agency_or_subsidiary
            }
            DisbursedReceivableAccountType::NonDomiciledCompany => term_type.non_domiciled_company,
        }
    }

    pub async fn create_accounts_for_credit_facility(
        &self,
        op: &mut cala_ledger::LedgerOperation<'_>,
        credit_facility_id: CreditFacilityId,
        account_ids: CreditFacilityAccountIds,
        disbursed_account_type: DisbursedReceivableAccountType,
    ) -> Result<(), CreditLedgerError> {
        let collateral_reference = &format!("credit-facility-collateral:{}", credit_facility_id);
        let collateral_name = &format!(
            "Credit Facility Collateral Account for {}",
            credit_facility_id
        );
        self.create_account_in_op(
            op,
            account_ids.collateral_account_id,
            self.internal_account_sets.collateral,
            collateral_reference,
            collateral_name,
            collateral_name,
        )
        .await?;

        let facility_reference = &format!("credit-facility-obs-facility:{}", credit_facility_id);
        let facility_name = &format!(
            "Off-Balance-Sheet Facility Account for Credit Facility {}",
            credit_facility_id
        );
        self.create_account_in_op(
            op,
            account_ids.facility_account_id,
            self.internal_account_sets.facility,
            facility_reference,
            facility_name,
            facility_name,
        )
        .await?;

        let disbursed_receivable_reference = &format!(
            "credit-facility-disbursed-receivable:{}",
            credit_facility_id
        );
        let disbursed_receivable_name = &format!(
            "Disbursed Receivable Account for Credit Facility {}",
            credit_facility_id
        );
        self.create_account_in_op(
            op,
            account_ids.disbursed_receivable_account_id,
            self.disbursed_internal_account_set_from_type(disbursed_account_type, true),
            disbursed_receivable_reference,
            disbursed_receivable_name,
            disbursed_receivable_name,
        )
        .await?;

        let interest_receivable_reference =
            &format!("credit-facility-interest-receivable:{}", credit_facility_id);
        let interest_receivable_name = &format!(
            "Interest Receivable Account for Credit Facility {}",
            credit_facility_id
        );
        self.create_account_in_op(
            op,
            account_ids.interest_receivable_account_id,
            self.internal_account_sets.interest_receivable,
            interest_receivable_reference,
            interest_receivable_name,
            interest_receivable_name,
        )
        .await?;

        let interest_income_reference =
            &format!("credit-facility-interest-income:{}", credit_facility_id);
        let interest_income_name = &format!(
            "Interest Income Account for Credit Facility {}",
            credit_facility_id
        );
        self.create_account_in_op(
            op,
            account_ids.interest_account_id,
            self.internal_account_sets.interest_income,
            interest_income_reference,
            interest_income_name,
            interest_income_name,
        )
        .await?;

        let fee_income_reference = &format!("credit-facility-fee-income:{}", credit_facility_id);
        let fee_income_name = &format!(
            "Fee Income Account for Credit Facility {}",
            credit_facility_id
        );
        self.create_account_in_op(
            op,
            account_ids.fee_income_account_id,
            self.internal_account_sets.fee_income,
            fee_income_reference,
            fee_income_name,
            fee_income_name,
        )
        .await?;

        Ok(())
    }

    pub async fn get_chart_of_accounts_integration_config(
        &self,
    ) -> Result<Option<ChartOfAccountsIntegrationConfig>, CreditLedgerError> {
        let account_set_id = *self
            .internal_account_sets
            .account_set_ids()
            .first()
            .expect("No internal account set ids found");
        let account_set = self.cala.account_sets().find(account_set_id).await?;
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
    ) -> Result<(), CreditLedgerError>
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
        charts_integration_meta: ChartOfAccountsIntegrationMeta,
    ) -> Result<(), CreditLedgerError> {
        let mut op = self.cala.begin_operation().await?;

        let mut account_set_ids = vec![
            self.facility_omnibus_account_ids.account_set_id,
            self.collateral_omnibus_account_ids.account_set_id,
        ];
        account_set_ids.extend(self.internal_account_sets.account_set_ids());
        let mut account_sets = self
            .cala
            .account_sets()
            .find_all_in_op::<AccountSet>(&mut op, &account_set_ids)
            .await?;

        let ChartOfAccountsIntegrationMeta {
            config: _,
            audit_info: _,

            facility_omnibus_parent_account_set_id,
            collateral_omnibus_parent_account_set_id,
            facility_parent_account_set_id,
            collateral_parent_account_set_id,
            interest_receivable_parent_account_set_id,
            interest_income_parent_account_set_id,
            fee_income_parent_account_set_id,
            ..
        } = &charts_integration_meta;

        self.attach_charts_account_set(
            &mut op,
            &mut account_sets,
            self.facility_omnibus_account_ids.account_set_id,
            *facility_omnibus_parent_account_set_id,
            &charts_integration_meta,
            |meta| meta.facility_omnibus_parent_account_set_id,
        )
        .await?;
        self.attach_charts_account_set(
            &mut op,
            &mut account_sets,
            self.collateral_omnibus_account_ids.account_set_id,
            *collateral_omnibus_parent_account_set_id,
            &charts_integration_meta,
            |meta| meta.collateral_omnibus_parent_account_set_id,
        )
        .await?;

        self.attach_charts_account_set(
            &mut op,
            &mut account_sets,
            self.internal_account_sets.facility.id,
            *facility_parent_account_set_id,
            &charts_integration_meta,
            |meta| meta.facility_parent_account_set_id,
        )
        .await?;
        self.attach_charts_account_set(
            &mut op,
            &mut account_sets,
            self.internal_account_sets.collateral.id,
            *collateral_parent_account_set_id,
            &charts_integration_meta,
            |meta| meta.collateral_parent_account_set_id,
        )
        .await?;
        self.attach_charts_account_set(
            &mut op,
            &mut account_sets,
            self.internal_account_sets.interest_receivable.id,
            *interest_receivable_parent_account_set_id,
            &charts_integration_meta,
            |meta| meta.interest_receivable_parent_account_set_id,
        )
        .await?;
        self.attach_charts_account_set(
            &mut op,
            &mut account_sets,
            self.internal_account_sets.interest_income.id,
            *interest_income_parent_account_set_id,
            &charts_integration_meta,
            |meta| meta.interest_income_parent_account_set_id,
        )
        .await?;
        self.attach_charts_account_set(
            &mut op,
            &mut account_sets,
            self.internal_account_sets.fee_income.id,
            *fee_income_parent_account_set_id,
            &charts_integration_meta,
            |meta| meta.fee_income_parent_account_set_id,
        )
        .await?;

        self.attach_short_term_disbursed_receivable_account_sets(
            &mut op,
            &mut account_sets,
            &charts_integration_meta,
        )
        .await?;
        self.attach_long_term_disbursed_receivable_account_sets(
            &mut op,
            &mut account_sets,
            &charts_integration_meta,
        )
        .await?;

        op.commit().await?;

        Ok(())
    }

    pub async fn attach_short_term_disbursed_receivable_account_sets(
        &self,
        op: &mut LedgerOperation<'_>,
        account_sets: &mut HashMap<LedgerAccountSetId, AccountSet>,
        charts_integration_meta: &ChartOfAccountsIntegrationMeta,
    ) -> Result<(), CreditLedgerError> {
        let short_term = &self.internal_account_sets.disbursed_receivable.short_term;

        self.attach_charts_account_set(
            op,
            account_sets,
            short_term.individual.id,
            charts_integration_meta
                .short_term_individual_disbursed_receivable_parent_account_set_id,
            charts_integration_meta,
            |meta| meta.short_term_individual_disbursed_receivable_parent_account_set_id,
        )
        .await?;

        self.attach_charts_account_set(
            op,
            account_sets,
            short_term.government_entity.id,
            charts_integration_meta
                .short_term_government_entity_disbursed_receivable_parent_account_set_id,
            charts_integration_meta,
            |meta| meta.short_term_government_entity_disbursed_receivable_parent_account_set_id,
        )
        .await?;

        self.attach_charts_account_set(
            op,
            account_sets,
            short_term.private_company.id,
            charts_integration_meta
                .short_term_private_company_disbursed_receivable_parent_account_set_id,
            charts_integration_meta,
            |meta| meta.short_term_private_company_disbursed_receivable_parent_account_set_id,
        )
        .await?;

        self.attach_charts_account_set(
            op,
            account_sets,
            short_term.bank.id,
            charts_integration_meta.short_term_bank_disbursed_receivable_parent_account_set_id,
            charts_integration_meta,
            |meta| meta.short_term_bank_disbursed_receivable_parent_account_set_id,
        )
        .await?;

        self.attach_charts_account_set(
            op,
            account_sets,
            short_term.financial_institution.id,
            charts_integration_meta
                .short_term_financial_institution_disbursed_receivable_parent_account_set_id,
            charts_integration_meta,
            |meta| meta.short_term_financial_institution_disbursed_receivable_parent_account_set_id,
        )
        .await?;

        self.attach_charts_account_set(
            op,
            account_sets,
            short_term.foreign_agency_or_subsidiary.id,
            charts_integration_meta
                .short_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_account_set_id,
            charts_integration_meta,
            |meta| meta.short_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_account_set_id,
        )
        .await?;

        self.attach_charts_account_set(
            op,
            account_sets,
            short_term.non_domiciled_company.id,
            charts_integration_meta
                .short_term_non_domiciled_company_disbursed_receivable_parent_account_set_id,
            charts_integration_meta,
            |meta| meta.short_term_non_domiciled_company_disbursed_receivable_parent_account_set_id,
        )
        .await?;

        Ok(())
    }

    pub async fn attach_long_term_disbursed_receivable_account_sets(
        &self,
        op: &mut LedgerOperation<'_>,
        account_sets: &mut HashMap<LedgerAccountSetId, AccountSet>,
        charts_integration_meta: &ChartOfAccountsIntegrationMeta,
    ) -> Result<(), CreditLedgerError> {
        let long_term = &self.internal_account_sets.disbursed_receivable.long_term;

        self.attach_charts_account_set(
            op,
            account_sets,
            long_term.individual.id,
            charts_integration_meta.long_term_individual_disbursed_receivable_parent_account_set_id,
            charts_integration_meta,
            |meta| meta.long_term_individual_disbursed_receivable_parent_account_set_id,
        )
        .await?;

        self.attach_charts_account_set(
            op,
            account_sets,
            long_term.government_entity.id,
            charts_integration_meta
                .long_term_government_entity_disbursed_receivable_parent_account_set_id,
            charts_integration_meta,
            |meta| meta.long_term_government_entity_disbursed_receivable_parent_account_set_id,
        )
        .await?;

        self.attach_charts_account_set(
            op,
            account_sets,
            long_term.private_company.id,
            charts_integration_meta
                .long_term_private_company_disbursed_receivable_parent_account_set_id,
            charts_integration_meta,
            |meta| meta.long_term_private_company_disbursed_receivable_parent_account_set_id,
        )
        .await?;

        self.attach_charts_account_set(
            op,
            account_sets,
            long_term.bank.id,
            charts_integration_meta.long_term_bank_disbursed_receivable_parent_account_set_id,
            charts_integration_meta,
            |meta| meta.long_term_bank_disbursed_receivable_parent_account_set_id,
        )
        .await?;

        self.attach_charts_account_set(
            op,
            account_sets,
            long_term.financial_institution.id,
            charts_integration_meta
                .long_term_financial_institution_disbursed_receivable_parent_account_set_id,
            charts_integration_meta,
            |meta| meta.long_term_financial_institution_disbursed_receivable_parent_account_set_id,
        )
        .await?;

        self.attach_charts_account_set(
            op,
            account_sets,
            long_term.foreign_agency_or_subsidiary.id,
            charts_integration_meta
                .long_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_account_set_id,
            charts_integration_meta,
            |meta| meta.long_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_account_set_id,
        )
        .await?;

        self.attach_charts_account_set(
            op,
            account_sets,
            long_term.non_domiciled_company.id,
            charts_integration_meta
                .long_term_non_domiciled_company_disbursed_receivable_parent_account_set_id,
            charts_integration_meta,
            |meta| meta.long_term_non_domiciled_company_disbursed_receivable_parent_account_set_id,
        )
        .await?;

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChartOfAccountsIntegrationMeta {
    pub config: ChartOfAccountsIntegrationConfig,
    pub audit_info: AuditInfo,

    pub facility_omnibus_parent_account_set_id: LedgerAccountSetId,
    pub collateral_omnibus_parent_account_set_id: LedgerAccountSetId,
    pub facility_parent_account_set_id: LedgerAccountSetId,
    pub collateral_parent_account_set_id: LedgerAccountSetId,
    pub interest_receivable_parent_account_set_id: LedgerAccountSetId,
    pub interest_income_parent_account_set_id: LedgerAccountSetId,
    pub fee_income_parent_account_set_id: LedgerAccountSetId,

    pub short_term_individual_disbursed_receivable_parent_account_set_id: LedgerAccountSetId,
    pub short_term_government_entity_disbursed_receivable_parent_account_set_id: LedgerAccountSetId,
    pub short_term_private_company_disbursed_receivable_parent_account_set_id: LedgerAccountSetId,
    pub short_term_bank_disbursed_receivable_parent_account_set_id: LedgerAccountSetId,
    pub short_term_financial_institution_disbursed_receivable_parent_account_set_id:
        LedgerAccountSetId,
    pub short_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_account_set_id:
        LedgerAccountSetId,
    pub short_term_non_domiciled_company_disbursed_receivable_parent_account_set_id:
        LedgerAccountSetId,

    pub long_term_individual_disbursed_receivable_parent_account_set_id: LedgerAccountSetId,
    pub long_term_government_entity_disbursed_receivable_parent_account_set_id: LedgerAccountSetId,
    pub long_term_private_company_disbursed_receivable_parent_account_set_id: LedgerAccountSetId,
    pub long_term_bank_disbursed_receivable_parent_account_set_id: LedgerAccountSetId,
    pub long_term_financial_institution_disbursed_receivable_parent_account_set_id:
        LedgerAccountSetId,
    pub long_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_account_set_id:
        LedgerAccountSetId,
    pub long_term_non_domiciled_company_disbursed_receivable_parent_account_set_id:
        LedgerAccountSetId,
}
