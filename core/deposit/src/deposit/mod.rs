mod entity;
pub mod error;
mod repo;

pub use entity::Deposit;
pub(super) use entity::*;
pub use repo::deposit_cursor::DepositsByCreatedAtCursor;
pub(super) use repo::*;
// use authz::PermissionCheck;

// use crate::{
//     audit::AuditInfo,
//     authorization::{Authorization, DepositAction, Object},
//     customer::Customers,
//     data_export::Export,
//     ledger::Ledger,
//     primitives::{CustomerId, DepositId, Subject, UsdCents},
// };

// pub use entity::*;
// use error::DepositError;
// pub use repo::{deposit_cursor::DepositsByCreatedAtCursor, DepositRepo};

// #[derive(Clone)]
// pub struct Deposits {
//     repo: DepositRepo,
//     customers: Customers,
//     ledger: Ledger,
//     authz: Authorization,
// }

// impl Deposits {
//     pub fn new(
//         pool: &sqlx::PgPool,
//         customers: &Customers,
//         ledger: &Ledger,
//         authz: &Authorization,
//         export: &Export,
//     ) -> Self {
//         let repo = DepositRepo::new(pool, export);
//         Self {
//             repo,
//             customers: customers.clone(),
//             ledger: ledger.clone(),
//             authz: authz.clone(),
//         }
//     }

//     pub fn repo(&self) -> &DepositRepo {
//         &self.repo
//     }

//     pub async fn subject_can_record(
//         &self,
//         sub: &Subject,
//         enforce: bool,
//     ) -> Result<Option<AuditInfo>, DepositError> {
//         Ok(self
//             .authz
//             .evaluate_permission(sub, Object::Deposit, DepositAction::Record, enforce)
//             .await?)
//     }

//     pub async fn record(
//         &self,
//         sub: &Subject,
//         customer_id: impl Into<CustomerId>,
//         amount: UsdCents,
//         reference: Option<String>,
//     ) -> Result<Deposit, DepositError> {
//         let audit_info = self
//             .subject_can_record(sub, true)
//             .await?
//             .expect("audit info missing");

//         let customer_id = customer_id.into();
//         let customer = self.customers.repo().find_by_id(customer_id).await?;
//         let new_deposit = NewDeposit::builder()
//             .id(DepositId::new())
//             .customer_id(customer_id)
//             .amount(amount)
//             .reference(reference.clone())
//             .credit_account_id(customer.account_ids.deposit_account_id)
//             .audit_info(audit_info)
//             .build()
//             .expect("Could not build Deposit");

//         let mut db = self.repo.begin_op().await?;
//         let deposit = self.repo.create_in_op(&mut db, new_deposit).await?;

//         self.ledger
//             .record_deposit_for_customer(
//                 deposit.id,
//                 customer.account_ids,
//                 amount,
//                 deposit.reference.clone(),
//             )
//             .await?;

//         db.commit().await?;

//         Ok(deposit)
//     }

//     pub async fn find_by_id(
//         &self,
//         sub: &Subject,
//         id: impl Into<DepositId> + std::fmt::Debug,
//     ) -> Result<Option<Deposit>, DepositError> {
//         self.authz
//             .enforce_permission(sub, Object::Deposit, DepositAction::Read)
//             .await?;

//         match self.repo.find_by_id(id.into()).await {
//             Ok(deposit) => Ok(Some(deposit)),
//             Err(DepositError::CouldNotFindById(_)) => Ok(None),
//             Err(e) => Err(e),
//         }
//     }

//     #[instrument(name = "deposit.list_for_customer", skip(self))]
//     pub async fn list_for_customer(
//         &self,
//         sub: &Subject,
//         customer_id: impl Into<CustomerId> + std::fmt::Debug,
//     ) -> Result<Vec<Deposit>, DepositError> {
//         let customer_id = customer_id.into();
//         self.authz
//             .enforce_permission(sub, Object::Deposit, DepositAction::List)
//             .await?;

//         Ok(self
//             .repo
//             .list_for_customer_id_by_created_at(
//                 customer_id,
//                 Default::default(),
//                 es_entity::ListDirection::Descending,
//             )
//             .await?
//             .entities)
//     }

//     pub async fn list(
//         &self,
//         sub: &Subject,
//         query: es_entity::PaginatedQueryArgs<DepositsByCreatedAtCursor>,
//     ) -> Result<es_entity::PaginatedQueryRet<Deposit, DepositsByCreatedAtCursor>, DepositError>
//     {
//         self.authz
//             .enforce_permission(sub, Object::Deposit, DepositAction::List)
//             .await?;
//         self.repo
//             .list_by_created_at(query, es_entity::ListDirection::Descending)
//             .await
//     }

//     pub async fn find_all<T: From<Deposit>>(
//         &self,
//         ids: &[DepositId],
//     ) -> Result<HashMap<DepositId, T>, DepositError> {
//         self.repo.find_all(ids).await
//     }
// }
