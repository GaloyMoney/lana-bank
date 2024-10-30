use async_graphql::dataloader::DataLoader;
use async_graphql::dataloader::Loader;

use std::collections::HashMap;

use lava_app::{app::LavaApp, user::error::UserError};

use crate::primitives::*;

use super::{approval_process::*, committee::Committee, policy::Policy, user::User};

pub type LavaDataLoader = DataLoader<LavaLoader>;
pub struct LavaLoader {
    pub app: LavaApp,
}

impl LavaLoader {
    pub fn new(app: &LavaApp) -> LavaDataLoader {
        DataLoader::new(Self { app: app.clone() }, tokio::task::spawn)
            // Set delay to 0 as per https://github.com/async-graphql/async-graphql/issues/1306
            .delay(std::time::Duration::from_secs(0))
    }
}

impl Loader<UserId> for LavaLoader {
    type Value = User;
    type Error = Arc<UserError>;

    async fn load(&self, keys: &[UserId]) -> Result<HashMap<UserId, User>, Self::Error> {
        self.app.users().find_all(keys).await.map_err(Arc::new)
    }
}

impl Loader<governance::CommitteeId> for LavaLoader {
    type Value = Committee;
    type Error = Arc<governance::committee_error::CommitteeError>;

    async fn load(
        &self,
        keys: &[governance::CommitteeId],
    ) -> Result<HashMap<governance::CommitteeId, Committee>, Self::Error> {
        self.app
            .governance()
            .find_all_committees(keys)
            .await
            .map_err(Arc::new)
    }
}

impl Loader<governance::PolicyId> for LavaLoader {
    type Value = Policy;
    type Error = Arc<governance::policy_error::PolicyError>;

    async fn load(
        &self,
        keys: &[governance::PolicyId],
    ) -> Result<HashMap<governance::PolicyId, Policy>, Self::Error> {
        self.app
            .governance()
            .find_all_policies(keys)
            .await
            .map_err(Arc::new)
    }
}

impl Loader<governance::ApprovalProcessId> for LavaLoader {
    type Value = ApprovalProcess;
    type Error = Arc<governance::approval_process_error::ApprovalProcessError>;

    async fn load(
        &self,
        keys: &[governance::ApprovalProcessId],
    ) -> Result<HashMap<governance::ApprovalProcessId, ApprovalProcess>, Self::Error> {
        self.app
            .governance()
            .find_all_approval_processes(keys)
            .await
            .map_err(Arc::new)
    }
}
