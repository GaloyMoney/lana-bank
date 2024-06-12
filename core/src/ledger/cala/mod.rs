mod convert;
pub mod error;
pub(super) mod graphql;

use cala_types::primitives::TxTemplateId;
use graphql_client::{GraphQLQuery, Response};
use reqwest::{Client as ReqwestClient, Method};
use rust_decimal::Decimal;
use tracing::instrument;
use uuid::Uuid;

use crate::primitives::{
    BfxAddressType, BfxIntegrationId, BfxWithdrawalMethod, LedgerAccountId, LedgerAccountSetId,
    LedgerAccountSetMemberType, LedgerDebitOrCredit, LedgerJournalId, LedgerTxId, WithdrawId,
};

use super::{
    bitfinex::BfxIntegration, fixed_term_loan::FixedTermLoanAccountIds, user::UserLedgerAccountIds,
};

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

    #[instrument(name = "lava.ledger.cala.find_journal_by_id", skip(self), err)]
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

    #[instrument(name = "lava.ledger.cala.create_core_journal", skip(self), err)]
    pub async fn create_core_journal(&self, id: Uuid) -> Result<LedgerJournalId, CalaError> {
        let variables = core_journal_create::Variables { id };
        let response =
            Self::traced_gql_request::<CoreJournalCreate, _>(&self.client, &self.url, variables)
                .await?;
        response
            .data
            .map(|d| LedgerJournalId::from(d.journal_create.journal.journal_id))
            .ok_or(CalaError::MissingDataField)
    }

    #[instrument(name = "lava.ledger.cala.create_account_set", skip(self), err)]
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

    #[instrument(name = "lava.ledger.cala.add_account_to_account_set", skip(self), err)]
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

    #[instrument(name = "lava.ledger.cala.find_account_set_by_id", skip(self, id), err)]
    pub async fn find_account_set_by_id<T: From<account_set_by_id::AccountSetByIdAccountSet>>(
        &self,
        id: impl Into<Uuid>,
    ) -> Result<Option<T>, CalaError> {
        let variables = account_set_by_id::Variables { id: id.into() };
        let response =
            Self::traced_gql_request::<AccountSetById, _>(&self.client, &self.url, variables)
                .await?;

        Ok(response.data.and_then(|d| d.account_set).map(T::from))
    }

    #[instrument(name = "lava.ledger.cala.create_account", skip(self), err)]
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

    #[instrument(name = "lava.ledger.cala.find_account_by_id", skip(self, id), err)]
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

    #[instrument(name = "lava.ledger.cala.find_by_id", skip(self), err)]
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

    #[instrument(name = "lava.ledger.cala.get_user_balance", skip(self), err)]
    pub async fn get_user_balance<T: From<user_balance::ResponseData>>(
        &self,
        account_ids: UserLedgerAccountIds,
    ) -> Result<Option<T>, CalaError> {
        let variables = user_balance::Variables {
            journal_id: super::constants::CORE_JOURNAL_ID,
            unallocated_collateral_id: Uuid::from(account_ids.unallocated_collateral_id),
            checking_id: Uuid::from(account_ids.checking_id),
        };
        let response =
            Self::traced_gql_request::<UserBalance, _>(&self.client, &self.url, variables).await?;

        Ok(response.data.map(T::from))
    }

    #[instrument(name = "lava.ledger.cala.get_fixed_term_loan_balance", skip(self), err)]
    pub async fn get_fixed_term_loan_balance<T: From<fixed_term_loan_balance::ResponseData>>(
        &self,
        account_ids: FixedTermLoanAccountIds,
    ) -> Result<Option<T>, CalaError> {
        let variables = fixed_term_loan_balance::Variables {
            journal_id: super::constants::CORE_JOURNAL_ID,
            collateral_id: Uuid::from(account_ids.collateral_account_id),
            loan_outstanding_id: Uuid::from(account_ids.outstanding_account_id),
            interest_income_id: Uuid::from(account_ids.interest_income_account_id),
        };
        let response =
            Self::traced_gql_request::<FixedTermLoanBalance, _>(&self.client, &self.url, variables)
                .await?;

        Ok(response.data.map(T::from))
    }

    #[instrument(name = "lava.ledger.cala.find_tx_template_by_code", skip(self), err)]
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
        name = "lava.ledger.cala.create_complete_loan_tx_template",
        skip(self),
        err
    )]
    pub async fn create_complete_loan_tx_template(
        &self,
        template_id: TxTemplateId,
    ) -> Result<TxTemplateId, CalaError> {
        let variables = complete_loan_template_create::Variables {
            template_id: Uuid::from(template_id),
            journal_id: format!("uuid(\"{}\")", super::constants::CORE_JOURNAL_ID),
        };
        let response = Self::traced_gql_request::<CompleteLoanTemplateCreate, _>(
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
        name = "lava.ledger.cala.create_initiate_withdrawal_from_checking_tx_template",
        skip(self),
        err
    )]
    pub async fn create_initiate_withdrawal_from_checking_tx_template(
        &self,
        template_id: TxTemplateId,
    ) -> Result<TxTemplateId, CalaError> {
        let variables = initiate_withdrawal_from_checking_tx_template_create::Variables {
            template_id: Uuid::from(template_id),
            journal_id: format!("uuid(\"{}\")", super::constants::CORE_JOURNAL_ID),
        };
        let response =
            Self::traced_gql_request::<InitiateWithdrawalFromCheckingTxTemplateCreate, _>(
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

    pub async fn execute_initiate_withdrawal_from_checking_tx(
        &self,
        user_account_ids: UserLedgerAccountIds,
        amount: Decimal,
        external_id: String,
    ) -> Result<(), CalaError> {
        let transaction_id = uuid::Uuid::new_v4();
        let variables = post_initiate_withdrawal_from_checking_transaction::Variables {
            transaction_id,
            user_account: user_account_ids.checking_id.into(),
            bank_account: super::constants::BANK_USDT_CASH_ID,
            amount,
            external_id,
        };
        let response =
            Self::traced_gql_request::<PostInitiateWithdrawalFromCheckingTransaction, _>(
                &self.client,
                &self.url,
                variables,
            )
            .await?;

        response
            .data
            .map(|d| d.post_transaction.transaction.transaction_id)
            .ok_or_else(|| CalaError::MissingDataField)?;
        Ok(())
    }

    #[instrument(
        name = "lava.ledger.cala.create_settle_withdrawal_from_checking_tx_template",
        skip(self),
        err
    )]
    pub async fn create_settle_withdrawal_from_checking_tx_template(
        &self,
        template_id: TxTemplateId,
    ) -> Result<TxTemplateId, CalaError> {
        let variables = settle_withdrawal_from_checking_tx_template_create::Variables {
            template_id: Uuid::from(template_id),
            journal_id: format!("uuid(\"{}\")", super::constants::CORE_JOURNAL_ID),
        };
        let response = Self::traced_gql_request::<SettleWithdrawalFromCheckingTxTemplateCreate, _>(
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

    pub async fn execute_settle_withdrawal_from_checking_tx(
        &self,
        user_account_ids: UserLedgerAccountIds,
        amount: Decimal,
        external_id: String,
    ) -> Result<(), CalaError> {
        let transaction_id = uuid::Uuid::new_v4();
        let variables = post_settle_withdrawal_from_checking_transaction::Variables {
            transaction_id,
            user_account: user_account_ids.checking_id.into(),
            bank_account: super::constants::BANK_USDT_CASH_ID,
            amount,
            external_id,
        };
        let response = Self::traced_gql_request::<PostSettleWithdrawalFromCheckingTransaction, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;

        response
            .data
            .map(|d| d.post_transaction.transaction.transaction_id)
            .ok_or_else(|| CalaError::MissingDataField)?;
        Ok(())
    }

    #[instrument(
        name = "lava.ledger.cala.create_approve_loan_template",
        skip(self),
        err
    )]
    pub async fn create_approve_loan_tx_template(
        &self,
        template_id: TxTemplateId,
    ) -> Result<TxTemplateId, CalaError> {
        let variables = approve_loan_template_create::Variables {
            template_id: Uuid::from(template_id),
            journal_id: format!("uuid(\"{}\")", super::constants::CORE_JOURNAL_ID),
        };
        let response = Self::traced_gql_request::<ApproveLoanTemplateCreate, _>(
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

    #[instrument(name = "lava.ledger.cala.execute_approve_loan_tx", skip(self), err)]
    pub async fn execute_approve_loan_tx(
        &self,
        transaction_id: LedgerTxId,
        loan_account_ids: FixedTermLoanAccountIds,
        user_account_ids: UserLedgerAccountIds,
        collateral_amount: Decimal,
        principal_amount: Decimal,
        external_id: String,
    ) -> Result<(), CalaError> {
        let variables = post_approve_loan_transaction::Variables {
            transaction_id: transaction_id.into(),
            unallocated_collateral_account: user_account_ids.unallocated_collateral_id.into(),
            loan_collateral_account: loan_account_ids.collateral_account_id.into(),
            loan_outstanding_account: loan_account_ids.outstanding_account_id.into(),
            checking_account: user_account_ids.checking_id.into(),
            collateral_amount,
            principal_amount,
            external_id,
        };
        let response = Self::traced_gql_request::<PostApproveLoanTransaction, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;

        response
            .data
            .map(|d| d.post_transaction.transaction.transaction_id)
            .ok_or_else(|| CalaError::MissingDataField)?;
        Ok(())
    }

    pub async fn execute_complete_loan_tx(
        &self,
        transaction_id: LedgerTxId,
        loan_account_ids: FixedTermLoanAccountIds,
        user_account_ids: UserLedgerAccountIds,
        payment_amount: Decimal,
        collateral_amount: Decimal,
        external_id: String,
    ) -> Result<(), CalaError> {
        let variables = post_complete_loan_transaction::Variables {
            transaction_id: transaction_id.into(),
            checking_account: user_account_ids.checking_id.into(),
            loan_outstanding_account: loan_account_ids.outstanding_account_id.into(),
            unallocated_collateral_account: user_account_ids.unallocated_collateral_id.into(),
            loan_collateral_account: loan_account_ids.collateral_account_id.into(),
            payment_amount,
            collateral_amount,
            external_id,
        };
        let response = Self::traced_gql_request::<PostCompleteLoanTransaction, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;

        response
            .data
            .map(|d| d.post_transaction.transaction.transaction_id)
            .ok_or_else(|| CalaError::MissingDataField)?;
        Ok(())
    }

    #[instrument(
        name = "lava.ledger.cala.create_incur_interest_template",
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
            .map(|d| d.tx_template_create.tx_template.tx_template_id)
            .map(TxTemplateId::from)
            .ok_or_else(|| CalaError::MissingDataField)
    }

    #[instrument(name = "lava.ledger.cala.execute_incur_interest_tx", skip(self), err)]
    pub async fn execute_incur_interest_tx(
        &self,
        transaction_id: LedgerTxId,
        loan_account_ids: FixedTermLoanAccountIds,
        interest_amount: Decimal,
        external_id: String,
    ) -> Result<(), CalaError> {
        let variables = post_incur_interest_transaction::Variables {
            transaction_id: transaction_id.into(),
            loan_outstanding_account: loan_account_ids.outstanding_account_id.into(),
            loan_interest_income_account: loan_account_ids.interest_income_account_id.into(),
            interest_amount,
            external_id,
        };
        let response = Self::traced_gql_request::<PostIncurInterestTransaction, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;

        response
            .data
            .map(|d| d.post_transaction.transaction.transaction_id)
            .ok_or_else(|| CalaError::MissingDataField)?;
        Ok(())
    }

    #[instrument(
        name = "lava.ledger.cala.create_record_payment_template",
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
            .map(|d| d.tx_template_create.tx_template.tx_template_id)
            .map(TxTemplateId::from)
            .ok_or_else(|| CalaError::MissingDataField)
    }

    #[instrument(name = "lava.ledger.cala.execute_repay_loan_tx", skip(self), err)]
    pub async fn execute_repay_loan_tx(
        &self,
        transaction_id: LedgerTxId,
        loan_account_ids: FixedTermLoanAccountIds,
        user_account_ids: UserLedgerAccountIds,
        payment_amount: Decimal,
        external_id: String,
    ) -> Result<(), CalaError> {
        let variables = post_record_payment_transaction::Variables {
            transaction_id: transaction_id.into(),
            checking_account: user_account_ids.checking_id.into(),
            loan_outstanding_account: loan_account_ids.outstanding_account_id.into(),
            payment_amount,
            external_id,
        };
        let response = Self::traced_gql_request::<PostRecordPaymentTransaction, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;

        response
            .data
            .map(|d| d.post_transaction.transaction.transaction_id)
            .ok_or_else(|| CalaError::MissingDataField)?;
        Ok(())
    }

    #[instrument(
        name = "lava.ledger.cala.execute_repay_loan_and_release_collateral_tx",
        skip(self),
        err
    )]
    pub async fn execute_repay_loan_and_release_collateral_tx(
        &self,
        transaction_id: LedgerTxId,
        loan_account_ids: FixedTermLoanAccountIds,
        user_account_ids: UserLedgerAccountIds,
        payment_amount: Decimal,
        external_id: String,
    ) -> Result<(), CalaError> {
        let variables = post_record_payment_transaction::Variables {
            transaction_id: transaction_id.into(),
            checking_account: user_account_ids.checking_id.into(),
            loan_outstanding_account: loan_account_ids.outstanding_account_id.into(),
            payment_amount,
            external_id,
        };
        let response = Self::traced_gql_request::<PostRecordPaymentTransaction, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;

        response
            .data
            .map(|d| d.post_transaction.transaction.transaction_id)
            .ok_or_else(|| CalaError::MissingDataField)?;
        Ok(())
    }

    // TODO: instrument and handle sensitive params
    pub async fn create_bfx_integration(
        &self,
        integration_id: BfxIntegrationId,
        name: String,
        key: String,
        secret: String,
    ) -> Result<BfxIntegration, CalaError> {
        let variables = bfx_integration_create::Variables {
            input: bfx_integration_create::BfxIntegrationCreateInput {
                integration_id: integration_id.into(),
                journal_id: super::constants::CORE_JOURNAL_ID,
                name,
                key,
                secret,
                description: None,
            },
        };
        let response =
            Self::traced_gql_request::<BfxIntegrationCreate, _>(&self.client, &self.url, variables)
                .await?;
        response
            .data
            .map(|d| BfxIntegration::from(d.bitfinex.integration_create.integration))
            .ok_or(CalaError::MissingDataField)
    }

    #[instrument(
        name = "lava.ledger.cala.find_bfx_integration_by_id",
        skip(self, id),
        err
    )]
    pub async fn find_bfx_integration_by_id<
        T: From<bfx_integration_by_id::BfxIntegrationByIdBitfinexIntegration>,
    >(
        &self,
        id: impl Into<Uuid>,
    ) -> Result<Option<T>, CalaError> {
        let variables = bfx_integration_by_id::Variables { id: id.into() };
        let response =
            Self::traced_gql_request::<BfxIntegrationById, _>(&self.client, &self.url, variables)
                .await?;

        Ok(response
            .data
            .and_then(|d| d.bitfinex.integration)
            .map(T::from))
    }

    #[instrument(
        name = "lava.ledger.cala.create_bfx_address_backed_account",
        skip(self),
        err
    )]
    pub async fn create_bfx_address_backed_account(
        &self,
        integration_id: BfxIntegrationId,
        address_type: BfxAddressType,
        account_id: LedgerAccountId,
        name: String,
        code: String,
        credit_account_id: LedgerAccountId,
    ) -> Result<String, CalaError> {
        let variables = bfx_address_backed_account_create::Variables {
            input: bfx_address_backed_account_create::BfxAddressBackedAccountCreateInput {
                account_id: account_id.into(),
                integration_id: integration_id.into(),
                type_: address_type.into(),
                deposit_credit_account_id: credit_account_id.into(),
                name,
                code,
                account_set_ids: None,
            },
        };
        let response = Self::traced_gql_request::<BfxAddressBackedAccountCreate, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;
        response
            .data
            .map(|d| d.bitfinex.address_backed_account_create.account.address)
            .ok_or(CalaError::MissingDataField)
    }

    #[instrument(name = "lava.ledger.cala.find_account_by_id", skip(self, id), err)]
    pub async fn find_address_backed_account_by_id<
        T: From<bfx_address_backed_account_by_id::BfxAddressBackedAccountByIdBitfinexAddressBackedAccount>,
    >(
        &self,
        id: impl Into<Uuid>,
    ) -> Result<Option<T>, CalaError>{
        let variables = bfx_address_backed_account_by_id::Variables { id: id.into() };
        let response = Self::traced_gql_request::<BfxAddressBackedAccountById, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;

        Ok(response
            .data
            .and_then(|d| d.bitfinex.address_backed_account)
            .map(T::from))
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn execute_bfx_withdrawal(
        &self,
        withdrawal_id: WithdrawId,
        integration_id: BfxIntegrationId,
        amount: Decimal,
        withdrawal_method: BfxWithdrawalMethod,
        destination_address: String,
        debit_account_id: LedgerAccountId,
        reserve_tx_external_id: String,
    ) -> Result<WithdrawId, CalaError> {
        let variables = bfx_withdrawal_execute::Variables {
            input: bfx_withdrawal_execute::BfxWithdrawalExecuteInput {
                withdrawal_id: withdrawal_id.into(),
                integration_id: integration_id.into(),
                amount,
                withdrawal_method: withdrawal_method.into(),
                destination_address,
                debit_account_id: debit_account_id.into(),
                reserve_tx_external_id,
            },
        };
        let response =
            Self::traced_gql_request::<BfxWithdrawalExecute, _>(&self.client, &self.url, variables)
                .await?;
        response
            .data
            .map(|d| d.bitfinex.withdrawal_execute.withdrawal.withdrawal_id)
            .map(WithdrawId::from)
            .ok_or(CalaError::MissingDataField)
    }

    async fn traced_gql_request<Q: GraphQLQuery, U: reqwest::IntoUrl>(
        client: &ReqwestClient,
        url: U,
        variables: Q::Variables,
    ) -> Result<Response<Q::ResponseData>, CalaError>
    where
        <Q as GraphQLQuery>::ResponseData: std::fmt::Debug,
    {
        let trace_headers = lava_tracing::http::inject_trace();
        let body = Q::build_query(variables);
        let response = client
            .request(Method::POST, url)
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
