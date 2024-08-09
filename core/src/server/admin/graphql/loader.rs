use async_graphql::dataloader::Loader;

use std::{collections::HashMap, sync::Arc};

use super::{audit::AuditEntry, user::User};
use crate::{
    app::LavaApp,
    audit::error::AuditError,
    customer::error::CustomerError,
    primitives::{AuditEntryId, CustomerId, UserId},
    server::shared_graphql::customer::Customer,
    user::error::UserError,
};

pub struct LavaDataLoader {
    pub app: LavaApp,
}

impl Loader<UserId> for LavaDataLoader {
    type Value = User;
    type Error = Arc<UserError>;

    async fn load(&self, keys: &[UserId]) -> Result<HashMap<UserId, User>, Self::Error> {
        self.app.users().find_all(keys).await.map_err(Arc::new)
    }
}

impl Loader<AuditEntryId> for LavaDataLoader {
    type Value = AuditEntry;
    type Error = Arc<AuditError>;

    async fn load(
        &self,
        keys: &[AuditEntryId],
    ) -> Result<HashMap<AuditEntryId, AuditEntry>, Self::Error> {
        self.app.audit().find_all(keys).await.map_err(Arc::new)
    }
}

impl Loader<CustomerId> for LavaDataLoader {
    type Value = Customer;
    type Error = Arc<CustomerError>;

    async fn load(
        &self,
        keys: &[CustomerId],
    ) -> Result<HashMap<CustomerId, Customer>, Self::Error> {
        self.app.customers().find_all(keys).await.map_err(Arc::new)
    }
}
