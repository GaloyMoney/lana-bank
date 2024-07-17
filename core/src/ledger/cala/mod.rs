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
    constants,
    loan::LoanAccountIds,
    user::{UserLedgerAccountAddresses, UserLedgerAccountIds},
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

    #[instrument(
        name = "lava.ledger.cala.find_account_set_and_sub_accounts_with_balance_by_id",
        skip(self, id),
        err
    )]
    pub async fn find_account_set_and_sub_accounts_with_balance_by_id<
        T: From<account_set_and_sub_accounts_with_balance::AccountSetAndSubAccountsWithBalanceAccountSet>,
    >(
        &self,
        id: impl Into<Uuid>,
        first: i64,
        after: Option<String>,
    ) -> Result<Option<T>, CalaError>{
        let variables = account_set_and_sub_accounts_with_balance::Variables {
            account_set_id: id.into(),
            journal_id: super::constants::CORE_JOURNAL_ID,
            first,
            after,
        };
        let response = Self::traced_gql_request::<AccountSetAndSubAccountsWithBalance, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;

        Ok(response.data.and_then(|d| d.account_set).map(T::from))
    }

    #[instrument(
        name = "lava.ledger.cala.find_account_set_and_sub_accounts_by_id",
        skip(self, id),
        err
    )]
    pub async fn find_account_set_and_sub_accounts_by_id<
        T: From<account_set_and_sub_accounts::AccountSetAndSubAccountsAccountSet>,
    >(
        &self,
        id: impl Into<Uuid>,
        first: i64,
        after: Option<String>,
    ) -> Result<Option<T>, CalaError> {
        let variables = account_set_and_sub_accounts::Variables {
            account_set_id: id.into(),
            first,
            after,
        };
        let response = Self::traced_gql_request::<AccountSetAndSubAccounts, _>(
            &self.client,
            &self.url,
            variables,
        )
        .await?;

        Ok(response.data.and_then(|d| d.account_set).map(T::from))
    }

    #[instrument(name = "lava.ledger.cala.create_user_accounts", skip(self), err)]
    pub async fn create_user_accounts(
        &self,
        user_id: impl Into<Uuid> + std::fmt::Debug,
        user_account_ids: UserLedgerAccountIds,
    ) -> Result<UserLedgerAccountAddresses, CalaError> {
        let user_id = user_id.into();
        let variables = create_user_accounts::Variables {
            on_balance_sheet_account_id: Uuid::from(
                user_account_ids.on_balance_sheet_deposit_account_id,
            ),
            on_balance_sheet_account_code: format!("USERS.CHECKING.{}", user_id),
            on_balance_sheet_account_name: format!("User Checking Account for {}", user_id),
            user_checking_control_account_set_id:
                super::constants::USER_CHECKING_CONTROL_ACCOUNT_SET_ID,
            tron_account_id: Uuid::new_v4(),
            tron_account_code: format!("ASSETS.TRON.{}", user_id),
            tron_account_name: format!("Bank USDT Deposit Account for {}", user_id),
            off_balance_sheet_account_id: Uuid::from(
                user_account_ids.off_balance_sheet_deposit_account_id,
            ),
            off_balance_sheet_account_code: format!("USERS.OFF_BALANCE_SHEET.{}", user_id),
            off_balance_sheet_account_name: format!(
                "Bank Off-Balance-Sheet Deposit Account for {}",
                user_id
            ),
            user_collateral_control_account_set_id:
                super::constants::USER_COLLATERAL_CONTROL_ACCOUNT_SET_ID,
            btc_account_id: Uuid::new_v4(),
            btc_account_code: format!("ASSETS.BTC.{}", user_id),
            btc_account_name: format!("Bank BTC Deposit Account for {}", user_id),
        };
        let response =
            Self::traced_gql_request::<CreateUserAccounts, _>(&self.client, &self.url, variables)
                .await?;
        response
            .data
            .map(|d| UserLedgerAccountAddresses {
                tron_usdt_address: d.tron_address.address_backed_account_create.account.address,
                btc_address: d.btc_address.address_backed_account_create.account.address,
            })
            .ok_or(CalaError::MissingDataField)
    }

    #[instrument(name = "lava.ledger.cala.create_user_accounts", skip(self), err)]
    pub async fn create_loan_accounts(
        &self,
        loan_id: impl Into<Uuid> + std::fmt::Debug,
        LoanAccountIds {
            collateral_account_id,
            outstanding_account_id,
            interest_account_id,
        }: LoanAccountIds,
    ) -> Result<(), CalaError> {
        let loan_id = loan_id.into();
        let variables = create_loan_accounts::Variables {
            loan_collateral_account_id: Uuid::from(collateral_account_id),
            loan_collateral_account_code: format!("LOANS.COLLATERAL.{}", loan_id),
            loan_collateral_account_name: format!("Loan Collateral Account for {}", loan_id),
            loans_collateral_control_account_set_id:
                super::constants::LOANS_COLLATERAL_CONTROL_ACCOUNT_SET_ID,
            loan_outstanding_account_id: Uuid::from(outstanding_account_id),
            loan_outstanding_account_code: format!("LOANS.RECEIVABLE.{}", loan_id),
            loan_outstanding_account_name: format!("Loan Receivable Account for {}", loan_id),
            loans_outstanding_control_account_set_id:
                super::constants::LOANS_RECEIVABLE_CONTROL_ACCOUNT_SET_ID,
            interest_account_id: Uuid::from(interest_account_id),
            interest_account_code: format!("LOANS.INTEREST_INCOME.{}", loan_id),
            interest_account_name: format!("Interest Income for Loan {}", loan_id),
            interest_revenue_control_account_set_id:
                super::constants::INTEREST_REVENUE_CONTROL_ACCOUNT_SET_ID,
        };
        let response =
            Self::traced_gql_request::<CreateLoanAccounts, _>(&self.client, &self.url, variables)
                .await?;
        response.data.ok_or(CalaError::MissingDataField)?;
        Ok(())
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
            off_balance_sheet_account_id: Uuid::from(
                account_ids.off_balance_sheet_deposit_account_id,
            ),
            on_balance_sheet_account_id: Uuid::from(
                account_ids.on_balance_sheet_deposit_account_id,
            ),
        };
        let response =
            Self::traced_gql_request::<UserBalance, _>(&self.client, &self.url, variables).await?;

        Ok(response.data.map(T::from))
    }

    #[instrument(name = "lava.ledger.cala.get_loan_balance", skip(self), err)]
    pub async fn get_loan_balance<T: From<loan_balance::ResponseData>>(
        &self,
        account_ids: LoanAccountIds,
    ) -> Result<Option<T>, CalaError> {
        let variables = loan_balance::Variables {
            journal_id: super::constants::CORE_JOURNAL_ID,
            collateral_id: Uuid::from(account_ids.collateral_account_id),
            loan_outstanding_id: Uuid::from(account_ids.outstanding_account_id),
            interest_income_id: Uuid::from(account_ids.interest_account_id),
        };
        let response =
            Self::traced_gql_request::<LoanBalance, _>(&self.client, &self.url, variables).await?;

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
            .map(|d| d.transaction_post.transaction.transaction_id)
            .ok_or_else(|| CalaError::MissingDataField)?;
        Ok(())
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
        loan_account_ids: LoanAccountIds,
        user_account_ids: UserLedgerAccountIds,
        collateral_amount: Decimal,
        principal_amount: Decimal,
        external_id: String,
    ) -> Result<(), CalaError> {
        let variables = post_approve_loan_transaction::Variables {
            transaction_id: transaction_id.into(),
            unallocated_collateral_account: user_account_ids
                .off_balance_sheet_deposit_account_id
                .into(),
            loan_collateral_account: loan_account_ids.collateral_account_id.into(),
            loan_outstanding_account: loan_account_ids.outstanding_account_id.into(),
            checking_account: user_account_ids.on_balance_sheet_deposit_account_id.into(),
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
            .map(|d| d.transaction_post.transaction.transaction_id)
            .ok_or_else(|| CalaError::MissingDataField)?;
        Ok(())
    }

    pub async fn execute_complete_loan_tx(
        &self,
        transaction_id: LedgerTxId,
        loan_account_ids: LoanAccountIds,
        user_account_ids: UserLedgerAccountIds,
        payment_amount: Decimal,
        collateral_amount: Decimal,
        external_id: String,
    ) -> Result<(), CalaError> {
        let variables = post_complete_loan_transaction::Variables {
            transaction_id: transaction_id.into(),
            checking_account: user_account_ids.on_balance_sheet_deposit_account_id.into(),
            loan_outstanding_account: loan_account_ids.outstanding_account_id.into(),
            unallocated_collateral_account: user_account_ids
                .off_balance_sheet_deposit_account_id
                .into(),
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
            .map(|d| d.transaction_post.transaction.transaction_id)
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
        loan_account_ids: LoanAccountIds,
        interest_amount: Decimal,
        external_id: String,
    ) -> Result<(), CalaError> {
        let variables = post_incur_interest_transaction::Variables {
            transaction_id: transaction_id.into(),
            loan_outstanding_account: loan_account_ids.outstanding_account_id.into(),
            loan_interest_income_account: loan_account_ids.interest_account_id.into(),
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
            .map(|d| d.transaction_post.transaction.transaction_id)
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
        loan_account_ids: LoanAccountIds,
        user_account_ids: UserLedgerAccountIds,
        payment_amount: Decimal,
        external_id: String,
    ) -> Result<(), CalaError> {
        let variables = post_record_payment_transaction::Variables {
            transaction_id: transaction_id.into(),
            checking_account: user_account_ids.on_balance_sheet_deposit_account_id.into(),
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
            .map(|d| d.transaction_post.transaction.transaction_id)
            .ok_or_else(|| CalaError::MissingDataField)?;
        Ok(())
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
                reserve_tx_external_id: Some(reserve_tx_external_id),
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

    pub async fn trial_balance<T: From<trial_balance::TrialBalanceAccountSet>>(
        &self,
    ) -> Result<Option<T>, CalaError> {
        let variables = trial_balance::Variables {
            journal_id: constants::CORE_JOURNAL_ID,
            account_set_id: constants::TRIAL_BALANCE_ACCOUNT_SET_ID,
        };
        let response =
            Self::traced_gql_request::<TrialBalance, _>(&self.client, &self.url, variables).await?;
        Ok(response.data.and_then(|d| d.account_set).map(T::from))
    }

    pub async fn obs_trial_balance<T: From<trial_balance::TrialBalanceAccountSet>>(
        &self,
    ) -> Result<Option<T>, CalaError> {
        let variables = trial_balance::Variables {
            journal_id: constants::CORE_JOURNAL_ID,
            account_set_id: constants::OBS_TRIAL_BALANCE_ACCOUNT_SET_ID,
        };
        let response =
            Self::traced_gql_request::<TrialBalance, _>(&self.client, &self.url, variables).await?;
        Ok(response.data.and_then(|d| d.account_set).map(T::from))
    }

    pub async fn chart_of_accounts<T: From<chart_of_accounts::ChartOfAccountsAccountSet>>(
        &self,
    ) -> Result<Option<T>, CalaError> {
        let variables = chart_of_accounts::Variables {
            account_set_id: constants::CHART_OF_ACCOUNTS_ACCOUNT_SET_ID,
        };
        let response =
            Self::traced_gql_request::<ChartOfAccounts, _>(&self.client, &self.url, variables)
                .await?;
        Ok(response.data.and_then(|d| d.account_set).map(T::from))
    }

    pub async fn obs_chart_of_accounts<T: From<chart_of_accounts::ChartOfAccountsAccountSet>>(
        &self,
    ) -> Result<Option<T>, CalaError> {
        let variables = chart_of_accounts::Variables {
            account_set_id: constants::OBS_CHART_OF_ACCOUNTS_ACCOUNT_SET_ID,
        };
        let response =
            Self::traced_gql_request::<ChartOfAccounts, _>(&self.client, &self.url, variables)
                .await?;
        Ok(response.data.and_then(|d| d.account_set).map(T::from))
    }

    #[instrument(name = "lava.ledger.cala.find_by_id", skip(self), err)]
    async fn find_account_by_code<T: From<account_by_code::AccountByCodeAccountByCode>>(
        &self,
        code: String,
    ) -> Result<Option<T>, CalaError> {
        let variables = account_by_code::Variables { code };
        let response =
            Self::traced_gql_request::<AccountByCode, _>(&self.client, &self.url, variables)
                .await?;

        Ok(response.data.and_then(|d| d.account_by_code).map(T::from))
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
