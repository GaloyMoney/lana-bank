#[macro_use]
mod macros;
mod error;

use crate::{customer::Customers, deposit::Deposits};
use core_customer::{CustomerId, CustomerType};
use core_deposit::{DepositAccountId, DepositId, UsdCents, WithdrawalId};
pub use error::CommandInboxError;

const COMMAND_INBOX_JOB: job::JobType = job::JobType::new("command-inbox");

// Define all async commands using the macro
// Adding a new command only requires adding an entry here!
define_async_commands! {
    handlers: {
        customers: Customers,
        deposits: Deposits,
    },
    commands: [
        {
            name: CreateCustomer,
            id_field: customer_id,
            id_type: CustomerId,
            result_type: core_customer::Customer,
            handler: customers,
            auth_check: subject_can_create_customer,
            execute_fn: create_with_id,
            find_fn: find_by_id_internal,
            not_found_error: CustomerNotFoundAfterProcessing,
            fields: {
                email: String,
                telegram_id: String,
                customer_type: CustomerType,
            },
        },
        {
            name: RecordDeposit,
            id_field: deposit_id,
            id_type: DepositId,
            result_type: core_deposit::Deposit,
            handler: deposits,
            auth_check: subject_can_record_deposit,
            execute_fn: record_deposit_with_id,
            find_fn: find_deposit_by_id_internal,
            not_found_error: DepositNotFoundAfterProcessing,
            fields: {
                deposit_account_id: DepositAccountId,
                amount: UsdCents,
                reference: Option<String>,
            },
        },
        {
            name: InitiateWithdrawal,
            id_field: withdrawal_id,
            id_type: WithdrawalId,
            result_type: core_deposit::Withdrawal,
            handler: deposits,
            auth_check: subject_can_initiate_withdrawal,
            execute_fn: initiate_withdrawal_with_id,
            find_fn: find_withdrawal_by_id_internal,
            not_found_error: WithdrawalNotFoundAfterProcessing,
            fields: {
                deposit_account_id: DepositAccountId,
                amount: UsdCents,
                reference: Option<String>,
            },
        },
    ]
}
