use chrono::{DateTime, Utc};

use core_money::Satoshis;

pub enum CustodianNotification {
    WalletBalanceChanged {
        external_wallet_id: String,
        new_balance: Satoshis,
        changed_at: DateTime<Utc>,
    },
}
