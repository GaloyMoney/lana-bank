mod convert;
pub mod error;
pub(super) mod graphql;

use cala_types::primitives::TxTemplateId;
use graphql_client::{GraphQLQuery, Response};
use reqwest::{Client as ReqwestClient, Method};
use rust_decimal::Decimal;
use tracing::instrument;
use uuid::Uuid;

use crate::primitives::{LedgerAccountId, LedgerDebitOrCredit, LedgerJournalId, LedgerTxId};

use super::{fixed_term_loan::FixedTermLoanAccountIds, user::UserLedgerAccountIds};

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
        name = "lava.ledger.cala.create_add_equity_tx_template",
        skip(self),
        err
    )]
    pub async fn create_add_equity_tx_template(
        &self,
        template_id: TxTemplateId,
    ) -> Result<TxTemplateId, CalaError> {
        let variables = add_equity_template_create::Variables {
            template_id: Uuid::from(template_id),
            journal_id: format!("uuid(\"{}\")", super::constants::CORE_JOURNAL_ID),
            bank_usd_account_id: format!("uuid(\"{}\")", super::constants::BANK_USDT_CASH_ID),
            bank_equity_account_id: format!(
                "uuid(\"{}\")",
                super::constants::BANK_SHAREHOLDER_EQUITY_ID
            ),
        };
        let response = Self::traced_gql_request::<AddEquityTemplateCreate, _>(
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
        name = "lava.ledger.cala.create_pledge_unallocated_collateral_tx_template",
        skip(self),
        err
    )]
    pub async fn create_pledge_unallocated_collateral_tx_template(
        &self,
        template_id: TxTemplateId,
    ) -> Result<TxTemplateId, CalaError> {
        let variables = pledge_unallocated_collateral_template_create::Variables {
            template_id: Uuid::from(template_id),
            journal_id: format!("uuid(\"{}\")", super::constants::CORE_JOURNAL_ID),
            asset_account_id: format!("uuid(\"{}\")", super::constants::BANK_OFF_BALANCE_SHEET_ID),
        };
        let response = Self::traced_gql_request::<PledgeUnallocatedCollateralTemplateCreate, _>(
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
        name = "lava.ledger.cala.create_deposit_checking_tx_template",
        skip(self),
        err
    )]
    pub async fn create_deposit_checking_tx_template(
        &self,
        template_id: TxTemplateId,
    ) -> Result<TxTemplateId, CalaError> {
        let variables = deposit_checking_template_create::Variables {
            template_id: Uuid::from(template_id),
            journal_id: format!("uuid(\"{}\")", super::constants::CORE_JOURNAL_ID),
            bank_usd_account_id: format!("uuid(\"{}\")", super::constants::BANK_USDT_CASH_ID),
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

    #[instrument(name = "lava.ledger.cala.execute_add_equity_tx", skip(self), err)]
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
            .map(|d| d.post_transaction.transaction.transaction_id)
            .ok_or_else(|| CalaError::MissingDataField)?;
        Ok(())
    }

    #[instrument(
        name = "lava.ledger.cala.execute_pledge_unallocated_collateral_tx",
        skip(self),
        err
    )]
    pub async fn execute_pledge_unallocated_collateral_tx(
        &self,
        account_id: LedgerAccountId,
        amount: Decimal,
        external_id: String,
    ) -> Result<(), CalaError> {
        let transaction_id = uuid::Uuid::new_v4();
        let variables = post_pledge_unallocated_collateral_transaction::Variables {
            transaction_id,
            account_id: Uuid::from(account_id),
            amount,
            external_id,
        };
        let response = Self::traced_gql_request::<PostPledgeUnallocatedCollateralTransaction, _>(
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

    #[instrument(name = "lava.ledger.cala.execute_deposit_checking_tx", skip(self), err)]
    pub async fn execute_deposit_checking_tx(
        &self,
        account_id: LedgerAccountId,
        amount: Decimal,
        external_id: String,
    ) -> Result<(), CalaError> {
        let transaction_id = uuid::Uuid::new_v4();
        let variables = post_deposit_checking_transaction::Variables {
            transaction_id,
            account_id: Uuid::from(account_id),
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
            .map(|d| d.post_transaction.transaction.transaction_id)
            .ok_or_else(|| CalaError::MissingDataField)?;
        Ok(())
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
