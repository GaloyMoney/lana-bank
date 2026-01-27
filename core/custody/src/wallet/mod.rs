mod entity;
pub mod error;
mod repo;

pub use entity::WalletBalance;
pub use entity::{NewWallet, Wallet, WalletEvent};
pub use repo::WalletRepo;
