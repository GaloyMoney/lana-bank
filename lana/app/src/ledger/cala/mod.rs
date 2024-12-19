mod convert;
pub mod error;
pub(super) mod graphql;

use cala_ledger::primitives::TxTemplateId;
use chrono::{DateTime, Utc};
use graphql_client::{GraphQLQuery, Response};
use reqwest::Client as ReqwestClient;
use rust_decimal::Decimal;
use tracing::instrument;
use uuid::Uuid;

use crate::primitives::{
    LedgerAccountId, LedgerAccountSetId, LedgerAccountSetMemberType, LedgerDebitOrCredit,
    LedgerJournalId, LedgerTxId,
};

use super::{constants, customer::CustomerLedgerAccountIds, CreditFacilityAccountIds};

use error::*;
use graphql::*;

#[derive(Clone)]
pub struct CalaClient {
    url: String,
    client: ReqwestClient,
}

impl CalaClient {
    pub fn new(url: String) -> Self {
        let client = ReqwestClient::new();
        CalaClient { client, url }
    }

    #[instrument(name = "lana.ledger.cala.find_journal_by_id", skip(self), err)]
    pub async fn find_journal_by_id(&self, id: Uuid) -> Result<LedgerJournalId, CalaError> {
        let variables = journal_by_id::Variables { id };
        let response =
            Self::traced_gql_request::<JournalById, _>(&self.client, &self.url, variables).await?;
        response
            .data
            .and_then(|d| d.journal)
            .map(|d| LedgerJournalId::from(d.journal_id))
            .ok_or(CalaError::MissingDataField)
    }

    #[instrument(name = "lana.ledger.cala.create_account_set", skip(self), err)]
    pub async fn create_account_set(
        &self,
        account_set_id: LedgerAccountSetId,
        name: String,
        normal_balance_type: LedgerDebitOrCredit,
    ) -> Result<LedgerAccountSetId, CalaError> {
        let variables = account_set_create::Variables {
            input: account_set_create::AccountSetCreateInput {
                journal_id: super::constants::CORE_JOURNAL_ID,
                account_set_id: Uuid::from(account_set_id),
                name,
                normal_balance_type: match normal_balance_type {
                    LedgerDebitOrCredit::Credit => account_set_create::DebitOrCredit::CREDIT,
                    LedgerDebitOrCredit::Debit => account_set_create::DebitOrCredit::DEBIT,
                },
                description: None,
                metadata: None,
            },
        };
        let response =
            Self::traced_gql_request::<AccountSetCreate, _>(&self.client, &self.url, variables)
                .await?;
        response
            .data
            .map(|d| LedgerAccountSetId::from(d.account_set_create.account_set.account_set_id))
            .ok_or(CalaError::MissingDataField)
    }

    #[instrument(name = "lana.ledger.cala.add_account_to_account_set", skip(self), err)]
    pub async fn add_account_to_account_set(
        &self,
        account_set_id: LedgerAccountSetId,
        member_id: LedgerAccountId,
    ) -> Result<LedgerAccountSetId, CalaError> {
        let variables = add_to_account_set::Variables {
            input: add_to_account_set::AddToAccountSetInput {
                account_set_id: account_set_id.into(),
                member_id: member_id.into(),
                member_type: LedgerAccountSetMemberType::Account.into(),
            },
        };
        let response =
            Self::traced_gql_request::<AddToAccountSet, _>(&self.client, &self.url, variables)
                .await?;
        response
            .data
            .map(|d| LedgerAccountSetId::from(d.add_to_account_set.account_set.account_set_id))
            .ok_or(CalaError::MissingDataField)
    }

    #[instrument(
        name = "lana.ledger.cala.find_account_set_and_sub_accounts_with_balance_by_id",
        skip(self, id),
        err
    )]
    pub async fn find_account_set_and_sub_accounts_with_balance_by_id<T, E>(
        &self,
        id: impl Into<Uuid>,
        first: i64,
        after: Option<String>,
        from: DateTime<Utc>,
        until: Option<DateTime<Utc>>
    ) -> Result<Option<T>, E>
    where
        T: TryFrom<account_set_and_sub_accounts_with_balance::AccountSetAndSubAccountsWithBalanceAccountSet, Error = E>,
        E: From<CalaError> + std::fmt::Display,
    {
        let variables = account_set_and_sub_accounts_with_balance::Variables {
            account_set_id: id.into(),
            journal_id: super::constants::CORE_JOURNAL_ID,
            first,
            after,
            from,
            until,
        };
        let response = Self::traced_gql_request::<AccountSetAndSubAccountsWithBalance, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;

        response
            .data
            .and_then(|d| d.account_set)
            .map(T::try_from)
            .transpose()
    }

    #[instrument(name = "lana.ledger.cala.create_user_accounts", skip(self), err)]
    pub async fn create_customer_accounts(
        &self,
        customer_id: impl Into<Uuid> + std::fmt::Debug,
        customer_account_ids: CustomerLedgerAccountIds,
    ) -> Result<(), CalaError> {
        let customer_id = customer_id.into();
        let variables = create_customer_accounts::Variables {
            on_balance_sheet_account_id: Uuid::from(
                customer_account_ids.on_balance_sheet_deposit_account_id,
            ),
            on_balance_sheet_account_code: format!("CUSTOMERS.CHECKING.{}", customer_id),
            on_balance_sheet_account_name: format!("Customer Checking Account for {}", customer_id),
            customer_checking_control_account_set_id:
                super::constants::CUSTOMER_CHECKING_CONTROL_ACCOUNT_SET_ID,
        };
        Self::traced_gql_request::<CreateCustomerAccounts, _>(&self.client, &self.url, variables)
            .await?;

        Ok(())
    }

    #[instrument(
        name = "lana.ledger.cala.create_credit_facility_accounts",
        skip(self),
        err
    )]
    pub async fn create_credit_facility_accounts(
        &self,
        credit_facility_id: impl Into<Uuid> + std::fmt::Debug,
        CreditFacilityAccountIds {
            facility_account_id,
            disbursed_receivable_account_id,
            collateral_account_id,
            interest_receivable_account_id,
            interest_account_id,
            fee_income_account_id,
        }: CreditFacilityAccountIds,
    ) -> Result<(), CalaError> {
        let credit_facility_id = credit_facility_id.into();
        let variables = create_credit_facility_accounts::Variables {
            facility_collateral_account_id: Uuid::from(collateral_account_id),
            facility_collateral_account_code: format!(
                "CREDIT_FACILITY.COLLATERAL.{}",
                credit_facility_id
            ),
            facility_collateral_account_name: format!(
                "Credit Facility Collateral Account for {}",
                credit_facility_id
            ),
            facilities_collateral_control_account_set_id:
                super::constants::OBS_CREDIT_FACILITIES_COLLATERAL_CONTROL_ACCOUNT_SET_ID,
            facility_account_id: Uuid::from(facility_account_id),
            facility_account_code: format!("CREDIT_FACILITY.OBS_FACILITY.{}", credit_facility_id),
            facility_account_name: format!(
                "Off-Balance-Sheet Facility Account for Credit Facility {}",
                credit_facility_id
            ),
            facilities_control_account_set_id:
                super::constants::OBS_CREDIT_FACILITY_CONTROL_ACCOUNT_SET_ID,
            facility_disbursed_receivable_account_id: Uuid::from(disbursed_receivable_account_id),
            facility_disbursed_receivable_account_code: format!(
                "CREDIT_FACILITY.DISBURSED_RECEIVABLE.{}",
                credit_facility_id
            ),
            facility_disbursed_receivable_account_name: format!(
                "Disbursed Receivable Account for Credit Facility {}",
                credit_facility_id
            ),
            facilities_disbursed_receivable_control_account_set_id:
                super::constants::CREDIT_FACILITIES_DISBURSED_RECEIVABLE_CONTROL_ACCOUNT_SET_ID,
            facility_interest_receivable_account_id: Uuid::from(interest_receivable_account_id),
            facility_interest_receivable_account_code: format!(
                "CREDIT_FACILITY.INTEREST_RECEIVABLE.{}",
                credit_facility_id
            ),
            facility_interest_receivable_account_name: format!(
                "Interest Receivable Account for Credit Facility {}",
                credit_facility_id
            ),
            facilities_interest_receivable_control_account_set_id:
                super::constants::CREDIT_FACILITIES_INTEREST_RECEIVABLE_CONTROL_ACCOUNT_SET_ID,
            interest_account_id: Uuid::from(interest_account_id),
            interest_account_code: format!(
                "CREDIT_FACILITY.INTEREST_INCOME.{}",
                credit_facility_id
            ),
            interest_account_name: format!(
                "Interest Income for Credit Facility {}",
                credit_facility_id
            ),
            interest_revenue_control_account_set_id:
                super::constants::CREDIT_FACILITY_INTEREST_REVENUE_CONTROL_ACCOUNT_SET_ID,
            fee_income_account_id: Uuid::from(fee_income_account_id),
            fee_income_account_code: format!("CREDIT_FACILITY.FEE_INCOME.{}", credit_facility_id),
            fee_income_account_name: format!(
                "Fee Income for Credit Facility {}",
                credit_facility_id
            ),
            fee_income_revenue_control_account_set_id:
                super::constants::CREDIT_FACILITY_FEE_REVENUE_CONTROL_ACCOUNT_SET_ID,
        };
        let response = Self::traced_gql_request::<CreateCreditFacilityAccounts, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;
        response.data.ok_or(CalaError::MissingDataField)?;
        Ok(())
    }

    #[instrument(name = "lana.ledger.cala.create_account", skip(self), err)]
    pub async fn create_account(
        &self,
        account_id: LedgerAccountId,
        normal_balance_type: LedgerDebitOrCredit,
        name: String,
        code: String,
        external_id: String,
    ) -> Result<LedgerAccountId, CalaError> {
        let variables = account_create::Variables {
            input: account_create::AccountCreateInput {
                account_id: Uuid::from(account_id),
                external_id: Some(external_id),
                normal_balance_type: match normal_balance_type {
                    LedgerDebitOrCredit::Credit => account_create::DebitOrCredit::CREDIT,
                    LedgerDebitOrCredit::Debit => account_create::DebitOrCredit::DEBIT,
                },
                status: account_create::Status::ACTIVE,
                name,
                code,
                description: None,
                metadata: None,
                account_set_ids: None,
            },
        };
        let response =
            Self::traced_gql_request::<AccountCreate, _>(&self.client, &self.url, variables)
                .await?;
        response
            .data
            .map(|d| LedgerAccountId::from(d.account_create.account.account_id))
            .ok_or(CalaError::MissingDataField)
    }

    #[instrument(name = "lana.ledger.cala.find_account_by_id", skip(self, id), err)]
    pub async fn find_account_by_id<T: From<account_by_id::AccountByIdAccount>>(
        &self,
        id: impl Into<Uuid>,
    ) -> Result<Option<T>, CalaError> {
        let variables = account_by_id::Variables {
            id: id.into(),
            journal_id: super::constants::CORE_JOURNAL_ID,
        };
        let response =
            Self::traced_gql_request::<AccountById, _>(&self.client, &self.url, variables).await?;

        Ok(response.data.and_then(|d| d.account).map(T::from))
    }

    #[instrument(name = "lana.ledger.cala.find_by_id", skip(self), err)]
    pub async fn find_account_by_external_id<
        T: From<account_by_external_id::AccountByExternalIdAccountByExternalId>,
    >(
        &self,
        external_id: String,
    ) -> Result<Option<T>, CalaError> {
        let variables = account_by_external_id::Variables {
            external_id,
            journal_id: super::constants::CORE_JOURNAL_ID,
        };
        let response =
            Self::traced_gql_request::<AccountByExternalId, _>(&self.client, &self.url, variables)
                .await?;

        Ok(response
            .data
            .and_then(|d| d.account_by_external_id)
            .map(T::from))
    }

    #[instrument(name = "lana.ledger.cala.get_customer_balance", skip(self), err)]
    pub async fn get_customer_balance<T, E>(
        &self,
        account_ids: CustomerLedgerAccountIds,
    ) -> Result<Option<T>, E>
    where
        T: TryFrom<customer_balance::ResponseData, Error = E>,
        E: From<CalaError> + std::fmt::Display,
    {
        let variables = customer_balance::Variables {
            journal_id: super::constants::CORE_JOURNAL_ID,
            on_balance_sheet_account_id: Uuid::from(
                account_ids.on_balance_sheet_deposit_account_id,
            ),
        };
        let response =
            Self::traced_gql_request::<CustomerBalance, _>(&self.client, &self.url, variables)
                .await?;

        response.data.map(T::try_from).transpose()
    }

    #[instrument(name = "lana.ledger.cala.get_credit_facility_balance", skip(self), err)]
    pub async fn get_credit_facility_balance<T, E>(
        &self,
        account_ids: CreditFacilityAccountIds,
    ) -> Result<Option<T>, E>
    where
        T: TryFrom<credit_facility_ledger_balance::ResponseData, Error = E>,
        E: From<CalaError> + std::fmt::Display,
    {
        let variables = credit_facility_ledger_balance::Variables {
            journal_id: super::constants::CORE_JOURNAL_ID,
            collateral_id: Uuid::from(account_ids.collateral_account_id),
            facility_id: Uuid::from(account_ids.facility_account_id),
            disbursed_receivable_id: Uuid::from(account_ids.disbursed_receivable_account_id),
            interest_receivable_id: Uuid::from(account_ids.interest_receivable_account_id),
        };
        let response = Self::traced_gql_request::<CreditFacilityLedgerBalance, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;

        response.data.map(T::try_from).transpose()
    }

    #[instrument(
        name = "lana.ledger.cala.create_facility_disbursal_tx_template",
        skip(self),
        err
    )]
    pub async fn create_facility_disbursal_tx_template(
        &self,
        template_id: TxTemplateId,
    ) -> Result<TxTemplateId, CalaError> {
        let obs_credit_facility_id = match Self::find_account_by_code::<LedgerAccountId>(
            self,
            super::constants::OBS_CREDIT_FACILITY_ACCOUNT_CODE.to_string(),
        )
        .await?
        {
            Some(id) => Ok(id),
            None => Err(CalaError::CouldNotFindAccountByCode(
                super::constants::OBS_CREDIT_FACILITY_ACCOUNT_CODE.to_string(),
            )),
        }?;
        let variables = credit_facility_disbursal_template_create::Variables {
            template_id: Uuid::from(template_id),
            journal_id: format!("uuid(\"{}\")", super::constants::CORE_JOURNAL_ID),
            omnibus_credit_facility_account: format!("uuid(\"{}\")", obs_credit_facility_id),
        };
        let response = Self::traced_gql_request::<CreditFacilityDisbursalTemplateCreate, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;

        response
            .data
            .map(|d| TxTemplateId::from(d.tx_template_create.tx_template.tx_template_id))
            .ok_or_else(|| CalaError::MissingDataField)
    }

    #[instrument(
        name = "lana.ledger.cala.execute_credit_facility_disbursal_tx",
        skip(self),
        err
    )]
    pub async fn execute_credit_facility_disbursal_tx(
        &self,
        transaction_id: LedgerTxId,
        facility_account_ids: CreditFacilityAccountIds,
        user_account_ids: CustomerLedgerAccountIds,
        disbursed_amount: Decimal,
        external_id: String,
    ) -> Result<chrono::DateTime<chrono::Utc>, CalaError> {
        let variables = post_credit_facility_disbursal_transaction::Variables {
            transaction_id: transaction_id.into(),
            facility_account: facility_account_ids.facility_account_id.into(),
            facility_disbursed_receivable_account: facility_account_ids
                .disbursed_receivable_account_id
                .into(),
            checking_account: user_account_ids.on_balance_sheet_deposit_account_id.into(),
            disbursed_amount,
            external_id,
        };
        let response = Self::traced_gql_request::<PostCreditFacilityDisbursalTransaction, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;

        let created_at = response
            .data
            .map(|d| d.transaction_post.transaction.created_at)
            .ok_or_else(|| CalaError::MissingDataField)?;
        Ok(created_at)
    }

    #[instrument(name = "lana.ledger.cala.find_tx_template_by_code", skip(self), err)]
    pub async fn find_tx_template_by_code<
        T: From<tx_template_by_code::TxTemplateByCodeTxTemplateByCode>,
    >(
        &self,
        code: String,
    ) -> Result<T, CalaError> {
        let variables = tx_template_by_code::Variables { code };
        let response =
            Self::traced_gql_request::<TxTemplateByCode, _>(&self.client, &self.url, variables)
                .await?;

        response
            .data
            .and_then(|d| d.tx_template_by_code)
            .map(T::from)
            .ok_or_else(|| CalaError::MissingDataField)
    }

    #[instrument(
        name = "lana.ledger.cala.create_add_equity_tx_template",
        skip(self),
        err
    )]
    pub async fn create_add_equity_tx_template(
        &self,
        template_id: TxTemplateId,
    ) -> Result<TxTemplateId, CalaError> {
        let bank_shareholder_equity_id = match Self::find_account_by_code::<LedgerAccountId>(
            self,
            super::constants::BANK_SHAREHOLDER_EQUITY_CODE.to_string(),
        )
        .await?
        {
            Some(id) => Ok(id),
            None => Err(CalaError::CouldNotFindAccountByCode(
                super::constants::BANK_SHAREHOLDER_EQUITY_CODE.to_string(),
            )),
        }?;

        let bank_reserve_id = match Self::find_account_by_code::<LedgerAccountId>(
            self,
            super::constants::BANK_RESERVE_FROM_SHAREHOLDER_CODE.to_string(),
        )
        .await?
        {
            Some(id) => Ok(id),
            None => Err(CalaError::CouldNotFindAccountByCode(
                super::constants::BANK_RESERVE_FROM_SHAREHOLDER_CODE.to_string(),
            )),
        }?;

        let variables = add_equity_template_create::Variables {
            template_id: Uuid::from(template_id),
            journal_id: format!("uuid(\"{}\")", super::constants::CORE_JOURNAL_ID),
            bank_reserve_account_id: format!("uuid(\"{}\")", bank_reserve_id),
            bank_equity_account_id: format!("uuid(\"{}\")", bank_shareholder_equity_id),
        };
        let response = Self::traced_gql_request::<AddEquityTemplateCreate, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;

        response
            .data
            .map(|d| TxTemplateId::from(d.tx_template_create.tx_template.tx_template_id))
            .ok_or_else(|| CalaError::MissingDataField)
    }

    #[instrument(name = "lana.ledger.cala.execute_add_equity_tx", skip(self), err)]
    pub async fn execute_add_equity_tx(
        &self,
        amount: Decimal,
        external_id: String,
    ) -> Result<(), CalaError> {
        let transaction_id = uuid::Uuid::new_v4();
        let variables = post_add_equity_transaction::Variables {
            transaction_id,
            amount,
            external_id,
        };
        let response = Self::traced_gql_request::<PostAddEquityTransaction, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;

        response
            .data
            .map(|d| d.transaction_post.transaction.transaction_id)
            .ok_or_else(|| CalaError::MissingDataField)?;
        Ok(())
    }

    #[instrument(
        name = "lana.ledger.cala.create_deposit_checking_tx_template",
        skip(self),
        err
    )]
    pub async fn create_deposit_checking_tx_template(
        &self,
        template_id: TxTemplateId,
    ) -> Result<TxTemplateId, CalaError> {
        let deposits_omnibus_id = match Self::find_account_by_code::<LedgerAccountId>(
            self,
            super::constants::BANK_DEPOSITS_OMNIBUS_CODE.to_string(),
        )
        .await?
        {
            Some(id) => Ok(id),
            None => Err(CalaError::CouldNotFindAccountByCode(
                super::constants::BANK_SHAREHOLDER_EQUITY_CODE.to_string(),
            )),
        }?;

        let variables = deposit_checking_template_create::Variables {
            template_id: Uuid::from(template_id),
            journal_id: format!("uuid(\"{}\")", super::constants::CORE_JOURNAL_ID),
            bank_usd_account_id: format!("uuid(\"{}\")", deposits_omnibus_id),
        };
        let response = Self::traced_gql_request::<DepositCheckingTemplateCreate, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;

        response
            .data
            .map(|d| d.tx_template_create.tx_template.tx_template_id)
            .map(TxTemplateId::from)
            .ok_or_else(|| CalaError::MissingDataField)
    }

    #[instrument(name = "lana.ledger.cala.execute_deposit_checking_tx", skip(self), err)]
    pub async fn execute_deposit_checking_tx(
        &self,
        transaction_id: LedgerTxId,
        customer_account_ids: CustomerLedgerAccountIds,
        amount: Decimal,
        external_id: String,
    ) -> Result<(), CalaError> {
        let variables = post_deposit_checking_transaction::Variables {
            transaction_id: transaction_id.into(),
            account_id: customer_account_ids
                .on_balance_sheet_deposit_account_id
                .into(),
            amount,
            external_id,
        };
        let response = Self::traced_gql_request::<PostDepositCheckingTransaction, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;

        response
            .data
            .map(|d| d.transaction_post.transaction.transaction_id)
            .ok_or_else(|| CalaError::MissingDataField)?;
        Ok(())
    }

    #[instrument(
        name = "lana.ledger.cala.create_initiate_withdraw_tx_template",
        skip(self),
        err
    )]
    pub async fn create_initiate_withdraw_tx_template(
        &self,
        template_id: TxTemplateId,
    ) -> Result<TxTemplateId, CalaError> {
        let deposits_omnibus_id = match Self::find_account_by_code::<LedgerAccountId>(
            self,
            super::constants::BANK_DEPOSITS_OMNIBUS_CODE.to_string(),
        )
        .await?
        {
            Some(id) => Ok(id),
            None => Err(CalaError::CouldNotFindAccountByCode(
                super::constants::BANK_SHAREHOLDER_EQUITY_CODE.to_string(),
            )),
        }?;

        let variables = initiate_withdraw_template_create::Variables {
            template_id: Uuid::from(template_id),
            journal_id: format!("uuid(\"{}\")", super::constants::CORE_JOURNAL_ID),
            bank_usd_account_id: format!("uuid(\"{}\")", deposits_omnibus_id),
        };
        let response = Self::traced_gql_request::<InitiateWithdrawTemplateCreate, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;

        response
            .data
            .map(|d| d.tx_template_create.tx_template.tx_template_id)
            .map(TxTemplateId::from)
            .ok_or_else(|| CalaError::MissingDataField)
    }

    #[instrument(
        name = "lana.ledger.cala.execute_initiate_withdraw_tx",
        skip(self),
        err
    )]
    pub async fn execute_initiate_withdraw_tx(
        &self,
        transaction_id: LedgerTxId,
        customer_account_ids: CustomerLedgerAccountIds,
        amount: Decimal,
        external_id: String,
    ) -> Result<(), CalaError> {
        let variables = post_initiate_withdraw_transaction::Variables {
            transaction_id: transaction_id.into(),
            account_id: customer_account_ids
                .on_balance_sheet_deposit_account_id
                .into(),
            amount,
            external_id,
        };
        let response = Self::traced_gql_request::<PostInitiateWithdrawTransaction, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;

        response
            .data
            .map(|d| d.transaction_post.transaction.transaction_id)
            .ok_or_else(|| CalaError::MissingDataField)?;
        Ok(())
    }

    #[instrument(
        name = "lana.ledger.cala.create_confirm_withdraw_tx_template",
        skip(self),
        err
    )]
    pub async fn create_confirm_withdraw_tx_template(
        &self,
        template_id: TxTemplateId,
    ) -> Result<TxTemplateId, CalaError> {
        let deposits_omnibus_id = match Self::find_account_by_code::<LedgerAccountId>(
            self,
            super::constants::BANK_DEPOSITS_OMNIBUS_CODE.to_string(),
        )
        .await?
        {
            Some(id) => Ok(id),
            None => Err(CalaError::CouldNotFindAccountByCode(
                super::constants::BANK_SHAREHOLDER_EQUITY_CODE.to_string(),
            )),
        }?;

        let variables = confirm_withdraw_template_create::Variables {
            template_id: Uuid::from(template_id),
            journal_id: format!("uuid(\"{}\")", super::constants::CORE_JOURNAL_ID),
            bank_usd_account_id: format!("uuid(\"{}\")", deposits_omnibus_id),
        };
        let response = Self::traced_gql_request::<ConfirmWithdrawTemplateCreate, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;

        response
            .data
            .map(|d| d.tx_template_create.tx_template.tx_template_id)
            .map(TxTemplateId::from)
            .ok_or_else(|| CalaError::MissingDataField)
    }

    #[instrument(name = "lana.ledger.cala.execute_confirm_withdraw_tx", skip(self), err)]
    pub async fn execute_confirm_withdraw_tx(
        &self,
        transaction_id: LedgerTxId,
        correlation_id: Uuid,
        debit_account_id: LedgerAccountId,
        amount: Decimal,
        external_id: String,
    ) -> Result<(), CalaError> {
        let variables = post_confirm_withdraw_transaction::Variables {
            transaction_id: transaction_id.into(),
            correlation_id: correlation_id.to_string(),
            account_id: debit_account_id.into(),
            amount,
            external_id,
        };
        let response = Self::traced_gql_request::<PostConfirmWithdrawTransaction, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;

        response
            .data
            .map(|d| d.transaction_post.transaction.transaction_id)
            .ok_or_else(|| CalaError::MissingDataField)?;
        Ok(())
    }

    #[instrument(
        name = "lana.ledger.cala.create_cancel_withdraw_tx_template",
        skip(self),
        err
    )]
    pub async fn create_cancel_withdraw_tx_template(
        &self,
        template_id: TxTemplateId,
    ) -> Result<TxTemplateId, CalaError> {
        let deposits_omnibus_id = match Self::find_account_by_code::<LedgerAccountId>(
            self,
            super::constants::BANK_DEPOSITS_OMNIBUS_CODE.to_string(),
        )
        .await?
        {
            Some(id) => Ok(id),
            None => Err(CalaError::CouldNotFindAccountByCode(
                super::constants::BANK_SHAREHOLDER_EQUITY_CODE.to_string(),
            )),
        }?;

        let variables = cancel_withdraw_template_create::Variables {
            template_id: Uuid::from(template_id),
            journal_id: format!("uuid(\"{}\")", super::constants::CORE_JOURNAL_ID),
            bank_usd_account_id: format!("uuid(\"{}\")", deposits_omnibus_id),
        };
        let response = Self::traced_gql_request::<CancelWithdrawTemplateCreate, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;

        response
            .data
            .map(|d| d.tx_template_create.tx_template.tx_template_id)
            .map(TxTemplateId::from)
            .ok_or_else(|| CalaError::MissingDataField)
    }

    #[instrument(name = "lana.ledger.cala.execute_cancel_withdraw_tx", skip(self), err)]
    pub async fn execute_cancel_withdraw_tx(
        &self,
        transaction_id: LedgerTxId,
        correlation_id: Uuid,
        debit_account_id: LedgerAccountId,
        amount: Decimal,
        external_id: String,
    ) -> Result<(), CalaError> {
        let variables = post_cancel_withdraw_transaction::Variables {
            transaction_id: transaction_id.into(),
            correlation_id: correlation_id.to_string(),
            account_id: debit_account_id.into(),
            amount,
            external_id,
        };
        let response = Self::traced_gql_request::<PostCancelWithdrawTransaction, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;

        response
            .data
            .map(|d| d.transaction_post.transaction.transaction_id)
            .ok_or_else(|| CalaError::MissingDataField)?;
        Ok(())
    }

    #[instrument(
        name = "lana.ledger.cala.create_approve_credit_facility_template",
        skip(self),
        err
    )]
    pub async fn create_approve_credit_facility_tx_template(
        &self,
        template_id: TxTemplateId,
    ) -> Result<TxTemplateId, CalaError> {
        let obs_credit_facility_id = match Self::find_account_by_code::<LedgerAccountId>(
            self,
            super::constants::OBS_CREDIT_FACILITY_ACCOUNT_CODE.to_string(),
        )
        .await?
        {
            Some(id) => Ok(id),
            None => Err(CalaError::CouldNotFindAccountByCode(
                super::constants::OBS_CREDIT_FACILITY_ACCOUNT_CODE.to_string(),
            )),
        }?;
        let variables = approve_credit_facility_template_create::Variables {
            template_id: Uuid::from(template_id),
            journal_id: format!("uuid(\"{}\")", super::constants::CORE_JOURNAL_ID),
            omnibus_credit_facility_account: format!("uuid(\"{}\")", obs_credit_facility_id),
        };
        let response = Self::traced_gql_request::<ApproveCreditFacilityTemplateCreate, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;

        response
            .data
            .map(|d| TxTemplateId::from(d.tx_template_create.tx_template.tx_template_id))
            .ok_or_else(|| CalaError::MissingDataField)
    }

    #[instrument(
        name = "lana.ledger.cala.execute_approve_credit_facility_tx",
        skip(self),
        err
    )]
    pub async fn execute_approve_credit_facility_tx(
        &self,
        transaction_id: LedgerTxId,
        credit_facility_account_ids: CreditFacilityAccountIds,
        user_account_ids: CustomerLedgerAccountIds,
        facility_amount: Decimal,
        structuring_fee_amount: Decimal,
        external_id: String,
    ) -> Result<chrono::DateTime<chrono::Utc>, CalaError> {
        let variables = post_approve_credit_facility_transaction::Variables {
            transaction_id: transaction_id.into(),
            credit_facility_account: credit_facility_account_ids.facility_account_id.into(),
            facility_disbursed_receivable_account: credit_facility_account_ids
                .disbursed_receivable_account_id
                .into(),
            facility_fee_income_account: credit_facility_account_ids.fee_income_account_id.into(),
            checking_account: user_account_ids.on_balance_sheet_deposit_account_id.into(),
            facility_amount,
            structuring_fee_amount,
            external_id,
        };
        let response = Self::traced_gql_request::<PostApproveCreditFacilityTransaction, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;

        let created_at = response
            .data
            .map(|d| d.transaction_post.transaction.created_at)
            .ok_or_else(|| CalaError::MissingDataField)?;
        Ok(created_at)
    }

    #[instrument(
        name = "lana.ledger.cala.create_add_collateral_tx_template",
        skip(self),
        err
    )]
    pub async fn create_add_collateral_tx_template(
        &self,
        template_id: TxTemplateId,
    ) -> Result<TxTemplateId, CalaError> {
        let obs_assets_id = match Self::find_account_by_code::<LedgerAccountId>(
            self,
            super::constants::OBS_ASSETS_ACCOUNT_CODE.to_string(),
        )
        .await?
        {
            Some(id) => Ok(id),
            None => Err(CalaError::CouldNotFindAccountByCode(
                super::constants::OBS_ASSETS_ACCOUNT_CODE.to_string(),
            )),
        }?;
        let variables = add_collateral_template_create::Variables {
            template_id: Uuid::from(template_id),
            journal_id: format!("uuid(\"{}\")", super::constants::CORE_JOURNAL_ID),
            bank_collateral_account_id: format!("uuid(\"{}\")", obs_assets_id),
        };
        let response = Self::traced_gql_request::<AddCollateralTemplateCreate, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;

        response
            .data
            .map(|d| d.tx_template_create.tx_template.tx_template_id)
            .map(TxTemplateId::from)
            .ok_or_else(|| CalaError::MissingDataField)
    }

    #[instrument(
        name = "lana.ledger.cala.add_credit_facility_collateral",
        skip(self),
        err
    )]
    pub async fn add_credit_facility_collateral(
        &self,
        transaction_id: LedgerTxId,
        credit_facility_account_ids: CreditFacilityAccountIds,
        collateral_amount: Decimal,
        external_id: String,
    ) -> Result<chrono::DateTime<chrono::Utc>, CalaError> {
        let variables = post_add_collateral_transaction::Variables {
            transaction_id: transaction_id.into(),
            collateral_account: credit_facility_account_ids.collateral_account_id.into(),
            collateral_amount,
            external_id,
        };
        let response = Self::traced_gql_request::<PostAddCollateralTransaction, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;

        let created_at = response
            .data
            .map(|d| d.transaction_post.transaction.created_at)
            .ok_or_else(|| CalaError::MissingDataField)?;
        Ok(created_at)
    }

    #[instrument(
        name = "lana.ledger.cala.create_remove_collateral_tx_template",
        skip(self),
        err
    )]
    pub async fn create_remove_collateral_tx_template(
        &self,
        template_id: TxTemplateId,
    ) -> Result<TxTemplateId, CalaError> {
        let obs_assets_id = match Self::find_account_by_code::<LedgerAccountId>(
            self,
            super::constants::OBS_ASSETS_ACCOUNT_CODE.to_string(),
        )
        .await?
        {
            Some(id) => Ok(id),
            None => Err(CalaError::CouldNotFindAccountByCode(
                super::constants::OBS_ASSETS_ACCOUNT_CODE.to_string(),
            )),
        }?;
        let variables = remove_collateral_template_create::Variables {
            template_id: Uuid::from(template_id),
            journal_id: format!("uuid(\"{}\")", super::constants::CORE_JOURNAL_ID),
            bank_collateral_account_id: format!("uuid(\"{}\")", obs_assets_id),
        };
        let response = Self::traced_gql_request::<RemoveCollateralTemplateCreate, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;

        response
            .data
            .map(|d| d.tx_template_create.tx_template.tx_template_id)
            .map(TxTemplateId::from)
            .ok_or_else(|| CalaError::MissingDataField)
    }

    #[instrument(
        name = "lana.ledger.cala.remove_credit_facility_collateral",
        skip(self),
        err
    )]
    pub async fn remove_credit_facility_collateral(
        &self,
        transaction_id: LedgerTxId,
        credit_facility_account_ids: CreditFacilityAccountIds,
        collateral_amount: Decimal,
        external_id: String,
    ) -> Result<chrono::DateTime<chrono::Utc>, CalaError> {
        let variables = post_remove_collateral_transaction::Variables {
            transaction_id: transaction_id.into(),
            collateral_account: credit_facility_account_ids.collateral_account_id.into(),
            collateral_amount,
            external_id,
        };
        let response = Self::traced_gql_request::<PostRemoveCollateralTransaction, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;

        let created_at = response
            .data
            .map(|d| d.transaction_post.transaction.created_at)
            .ok_or_else(|| CalaError::MissingDataField)?;
        Ok(created_at)
    }

    #[instrument(
        name = "lana.ledger.cala.create_incur_interest_template",
        skip(self),
        err
    )]
    pub async fn create_incur_interest_tx_template(
        &self,
        template_id: TxTemplateId,
    ) -> Result<TxTemplateId, CalaError> {
        let variables = incur_interest_template_create::Variables {
            template_id: Uuid::from(template_id),
            journal_id: format!("uuid(\"{}\")", super::constants::CORE_JOURNAL_ID),
        };
        let response = Self::traced_gql_request::<IncurInterestTemplateCreate, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;

        response
            .data
            .map(|d| TxTemplateId::from(d.tx_template_create.tx_template.tx_template_id))
            .ok_or_else(|| CalaError::MissingDataField)
    }

    #[instrument(
        name = "lana.ledger.cala.create_credit_facility_incur_interest_template",
        skip(self),
        err
    )]
    pub async fn create_credit_facility_incur_interest_tx_template(
        &self,
        template_id: TxTemplateId,
    ) -> Result<TxTemplateId, CalaError> {
        let variables = credit_facility_incur_interest_template_create::Variables {
            template_id: Uuid::from(template_id),
            journal_id: format!("uuid(\"{}\")", super::constants::CORE_JOURNAL_ID),
        };
        let response = Self::traced_gql_request::<CreditFacilityIncurInterestTemplateCreate, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;

        response
            .data
            .map(|d| TxTemplateId::from(d.tx_template_create.tx_template.tx_template_id))
            .ok_or_else(|| CalaError::MissingDataField)
    }

    #[instrument(
        name = "lana.ledger.cala.execute_credit_facility_incur_interest_tx",
        skip(self),
        err
    )]
    pub async fn execute_credit_facility_incur_interest_tx(
        &self,
        transaction_id: LedgerTxId,
        facility_account_ids: CreditFacilityAccountIds,
        interest_amount: Decimal,
        external_id: String,
        effective: DateTime<Utc>,
    ) -> Result<chrono::DateTime<chrono::Utc>, CalaError> {
        let variables = post_credit_facility_incur_interest_transaction::Variables {
            transaction_id: transaction_id.into(),
            credit_facility_interest_receivable_account: facility_account_ids
                .interest_receivable_account_id
                .into(),
            credit_facility_interest_income_account: facility_account_ids
                .interest_account_id
                .into(),
            interest_amount,
            external_id,
            effective: effective.format("%Y-%m-%d").to_string(),
        };
        let response = Self::traced_gql_request::<PostCreditFacilityIncurInterestTransaction, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;

        let created_at = response
            .data
            .map(|d| d.transaction_post.transaction.created_at)
            .ok_or_else(|| CalaError::MissingDataField)?;
        Ok(created_at)
    }

    #[instrument(
        name = "lana.ledger.cala.create_credit_facility_accrue_interest_template",
        skip(self),
        err
    )]
    pub async fn create_credit_facility_accrue_interest_tx_template(
        &self,
        template_id: TxTemplateId,
    ) -> Result<TxTemplateId, CalaError> {
        let variables = credit_facility_accrue_interest_template_create::Variables {
            template_id: Uuid::from(template_id),
            journal_id: format!("uuid(\"{}\")", super::constants::CORE_JOURNAL_ID),
        };
        let response = Self::traced_gql_request::<CreditFacilityAccrueInterestTemplateCreate, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;

        response
            .data
            .map(|d| TxTemplateId::from(d.tx_template_create.tx_template.tx_template_id))
            .ok_or_else(|| CalaError::MissingDataField)
    }

    #[instrument(
        name = "lana.ledger.cala.execute_credit_facility_accrue_interest_tx",
        skip(self),
        err
    )]
    pub async fn execute_credit_facility_accrue_interest_tx(
        &self,
        transaction_id: LedgerTxId,
        facility_account_ids: CreditFacilityAccountIds,
        interest_amount: Decimal,
        external_id: String,
        effective: DateTime<Utc>,
    ) -> Result<chrono::DateTime<chrono::Utc>, CalaError> {
        let variables = post_credit_facility_accrue_interest_transaction::Variables {
            transaction_id: transaction_id.into(),
            credit_facility_interest_receivable_account: facility_account_ids
                .interest_receivable_account_id
                .into(),
            credit_facility_interest_income_account: facility_account_ids
                .interest_account_id
                .into(),
            interest_amount,
            external_id,
            effective: effective.format("%Y-%m-%d").to_string(),
        };
        let response = Self::traced_gql_request::<PostCreditFacilityAccrueInterestTransaction, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;

        let created_at = response
            .data
            .map(|d| d.transaction_post.transaction.created_at)
            .ok_or_else(|| CalaError::MissingDataField)?;
        Ok(created_at)
    }

    #[instrument(
        name = "lana.ledger.cala.create_record_payment_template",
        skip(self),
        err
    )]
    pub async fn create_record_payment_tx_template(
        &self,
        template_id: TxTemplateId,
    ) -> Result<TxTemplateId, CalaError> {
        let variables = record_payment_template_create::Variables {
            template_id: Uuid::from(template_id),
            journal_id: format!("uuid(\"{}\")", super::constants::CORE_JOURNAL_ID),
        };
        let response = Self::traced_gql_request::<RecordPaymentTemplateCreate, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;

        response
            .data
            .map(|d| TxTemplateId::from(d.tx_template_create.tx_template.tx_template_id))
            .ok_or_else(|| CalaError::MissingDataField)
    }

    pub async fn create_record_credit_facility_payment_tx_template(
        &self,
        template_id: TxTemplateId,
    ) -> Result<TxTemplateId, CalaError> {
        let variables = record_credit_facility_payment_template_create::Variables {
            template_id: Uuid::from(template_id),
            journal_id: format!("uuid(\"{}\")", super::constants::CORE_JOURNAL_ID),
        };
        let response = Self::traced_gql_request::<RecordCreditFacilityPaymentTemplateCreate, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;

        response
            .data
            .map(|d| TxTemplateId::from(d.tx_template_create.tx_template.tx_template_id))
            .ok_or_else(|| CalaError::MissingDataField)
    }

    #[instrument(
        name = "lana.ledger.cala.execute_repay_credit_facility_tx",
        skip(self),
        err
    )]
    pub async fn execute_repay_credit_facility_tx(
        &self,
        transaction_id: LedgerTxId,
        credit_facility_account_ids: CreditFacilityAccountIds,
        user_account_ids: CustomerLedgerAccountIds,
        interest_payment_amount: Decimal,
        disbursal_payment_amount: Decimal,
        external_id: String,
    ) -> Result<chrono::DateTime<chrono::Utc>, CalaError> {
        let variables = post_record_credit_facility_payment_transaction::Variables {
            transaction_id: transaction_id.into(),
            checking_account: user_account_ids.on_balance_sheet_deposit_account_id.into(),
            facility_interest_receivable_account: credit_facility_account_ids
                .interest_receivable_account_id
                .into(),
            facility_disbursed_receivable_account: credit_facility_account_ids
                .disbursed_receivable_account_id
                .into(),
            interest_payment_amount,
            disbursal_payment_amount,
            external_id,
        };
        let response = Self::traced_gql_request::<PostRecordCreditFacilityPaymentTransaction, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;

        let created_at = response
            .data
            .map(|d| d.transaction_post.transaction.created_at)
            .ok_or_else(|| CalaError::MissingDataField)?;
        Ok(created_at)
    }

    pub async fn trial_balance<T, E>(
        &self,
        from: DateTime<Utc>,
        until: Option<DateTime<Utc>>,
    ) -> Result<Option<T>, E>
    where
        T: TryFrom<trial_balance::TrialBalanceAccountSet, Error = E>,
        E: From<CalaError> + std::fmt::Display,
    {
        let variables = trial_balance::Variables {
            journal_id: constants::CORE_JOURNAL_ID,
            account_set_id: constants::TRIAL_BALANCE_ACCOUNT_SET_ID,
            from,
            until,
        };
        let response =
            Self::traced_gql_request::<TrialBalance, _>(&self.client, &self.url, variables).await?;
        response
            .data
            .and_then(|d| d.account_set)
            .map(T::try_from)
            .transpose()
    }

    pub async fn obs_trial_balance<T, E>(
        &self,
        from: DateTime<Utc>,
        until: Option<DateTime<Utc>>,
    ) -> Result<Option<T>, E>
    where
        T: TryFrom<trial_balance::TrialBalanceAccountSet, Error = E>,
        E: From<CalaError> + std::fmt::Display,
    {
        let variables = trial_balance::Variables {
            journal_id: constants::CORE_JOURNAL_ID,
            account_set_id: constants::OBS_TRIAL_BALANCE_ACCOUNT_SET_ID,
            from,
            until,
        };
        let response =
            Self::traced_gql_request::<TrialBalance, _>(&self.client, &self.url, variables).await?;
        response
            .data
            .and_then(|d| d.account_set)
            .map(T::try_from)
            .transpose()
    }

    pub async fn chart_of_accounts<T, E>(&self) -> Result<Option<T>, E>
    where
        T: TryFrom<chart_of_accounts::ChartOfAccountsAccountSet, Error = E>,
        E: From<CalaError> + std::fmt::Display,
    {
        let variables = chart_of_accounts::Variables {
            account_set_id: constants::CHART_OF_ACCOUNTS_ACCOUNT_SET_ID,
            journal_id: constants::CORE_JOURNAL_ID,
            from: Utc::now(),
            until: None,
        };
        let response =
            Self::traced_gql_request::<ChartOfAccounts, _>(&self.client, &self.url, variables)
                .await?;
        response
            .data
            .and_then(|d| d.account_set)
            .map(T::try_from)
            .transpose()
    }

    pub async fn obs_chart_of_accounts<T, E>(&self) -> Result<Option<T>, E>
    where
        T: TryFrom<chart_of_accounts::ChartOfAccountsAccountSet, Error = E>,
        E: From<CalaError> + std::fmt::Display,
    {
        let variables = chart_of_accounts::Variables {
            account_set_id: constants::OBS_CHART_OF_ACCOUNTS_ACCOUNT_SET_ID,
            journal_id: constants::CORE_JOURNAL_ID,
            from: Utc::now(),
            until: None,
        };
        let response =
            Self::traced_gql_request::<ChartOfAccounts, _>(&self.client, &self.url, variables)
                .await?;
        response
            .data
            .and_then(|d| d.account_set)
            .map(T::try_from)
            .transpose()
    }

    pub async fn balance_sheet<T, E>(
        &self,
        from: DateTime<Utc>,
        until: Option<DateTime<Utc>>,
    ) -> Result<Option<T>, E>
    where
        T: TryFrom<balance_sheet::BalanceSheetAccountSet, Error = E>,
        E: From<CalaError> + std::fmt::Display,
    {
        let variables = balance_sheet::Variables {
            account_set_id: constants::BALANCE_SHEET_ACCOUNT_SET_ID,
            journal_id: constants::CORE_JOURNAL_ID,
            from,
            until,
        };
        let response =
            Self::traced_gql_request::<BalanceSheet, _>(&self.client, &self.url, variables).await?;
        response
            .data
            .and_then(|d| d.account_set)
            .map(T::try_from)
            .transpose()
    }

    pub async fn profit_and_loss<T, E>(
        &self,
        from: DateTime<Utc>,
        until: Option<DateTime<Utc>>,
    ) -> Result<Option<T>, E>
    where
        T: TryFrom<profit_and_loss_statement::ProfitAndLossStatementAccountSet, Error = E>,
        E: From<CalaError> + std::fmt::Display,
    {
        let variables = profit_and_loss_statement::Variables {
            account_set_id: constants::NET_INCOME_ACCOUNT_SET_ID,
            journal_id: constants::CORE_JOURNAL_ID,
            from,
            until,
        };
        let response = Self::traced_gql_request::<ProfitAndLossStatement, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;
        response
            .data
            .and_then(|d| d.account_set)
            .map(T::try_from)
            .transpose()
    }

    pub async fn cash_flow<T, E>(
        &self,
        from: DateTime<Utc>,
        until: Option<DateTime<Utc>>,
    ) -> Result<Option<T>, E>
    where
        T: TryFrom<cash_flow_statement::CashFlowStatementAccountSet, Error = E>,
        E: From<CalaError> + std::fmt::Display,
    {
        let variables = cash_flow_statement::Variables {
            account_set_id: constants::CASH_FLOW_ACCOUNT_SET_ID,
            journal_id: constants::CORE_JOURNAL_ID,
            from,
            until,
        };
        let response =
            Self::traced_gql_request::<CashFlowStatement, _>(&self.client, &self.url, variables)
                .await?;
        response
            .data
            .and_then(|d| d.account_set)
            .map(T::try_from)
            .transpose()
    }

    #[instrument(name = "lana.ledger.cala.find_account_by_code", skip(self), err)]
    async fn find_account_by_code<T: From<account_by_code::AccountByCodeAccountByCode>>(
        &self,
        code: String,
    ) -> Result<Option<T>, CalaError> {
        let variables = account_by_code::Variables {
            code,
            journal_id: super::constants::CORE_JOURNAL_ID,
        };
        let response =
            Self::traced_gql_request::<AccountByCode, _>(&self.client, &self.url, variables)
                .await?;

        Ok(response.data.and_then(|d| d.account_by_code).map(T::from))
    }

    #[instrument(
        name = "lana.ledger.cala.find_account_with_balance_by_code",
        skip(self),
        err
    )]
    pub async fn find_account_with_balance_by_code<T, E>(
        &self,
        code: String,
    ) -> Result<Option<T>, E>
    where
        T: TryFrom<account_by_code::AccountByCodeAccountByCode, Error = E>,
        E: From<CalaError> + std::fmt::Display,
    {
        let variables = account_by_code::Variables {
            code,
            journal_id: super::constants::CORE_JOURNAL_ID,
        };
        let response =
            Self::traced_gql_request::<AccountByCode, _>(&self.client, &self.url, variables)
                .await?;

        response
            .data
            .and_then(|d| d.account_by_code)
            .map(T::try_from)
            .transpose()
    }

    async fn traced_gql_request<Q: GraphQLQuery, U: reqwest::IntoUrl>(
        client: &ReqwestClient,
        url: U,
        variables: Q::Variables,
    ) -> Result<Response<Q::ResponseData>, CalaError>
    where
        <Q as GraphQLQuery>::ResponseData: std::fmt::Debug,
    {
        let trace_headers = tracing_utils::http::inject_trace();
        let body = Q::build_query(variables);
        let response = client
            .post(url)
            .headers(trace_headers)
            .json(&body)
            .send()
            .await?;
        let response = response.json::<Response<Q::ResponseData>>().await?;

        if let Some(errors) = response.errors {
            return Err(CalaError::from(errors));
        }

        Ok(response)
    }
}
