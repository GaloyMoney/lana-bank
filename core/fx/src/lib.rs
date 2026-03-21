#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub(crate) mod chart_of_accounts_integration;
pub mod error;
pub(crate) mod ledger;
mod primitives;

use std::sync::Arc;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::CalaLedger;
use domain_config::InternalDomainConfigs;
use tracing_macros::record_error_severity;

use chart_of_accounts_integration::ChartOfAccountsIntegrations;
pub use chart_of_accounts_integration::{
    ChartOfAccountsIntegrationConfig, error::ChartOfAccountsIntegrationError,
};
use ledger::FxLedger;
pub use primitives::*;

pub struct CoreFx<Perms>
where
    Perms: PermissionCheck,
{
    ledger: Arc<FxLedger>,
    chart_of_accounts_integrations: Arc<ChartOfAccountsIntegrations<Perms>>,
}

impl<Perms> Clone for CoreFx<Perms>
where
    Perms: PermissionCheck,
{
    fn clone(&self) -> Self {
        Self {
            ledger: self.ledger.clone(),
            chart_of_accounts_integrations: self.chart_of_accounts_integrations.clone(),
        }
    }
}

impl<Perms> CoreFx<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreFxAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreFxObject>,
{
    #[record_error_severity]
    #[tracing::instrument(name = "fx.init", skip_all, fields(journal_id = %journal_id))]
    pub async fn init(
        authz: &Perms,
        cala: &CalaLedger,
        journal_id: CalaJournalId,
        jobs: &job::Jobs,
        internal_domain_configs: &InternalDomainConfigs,
    ) -> Result<Self, error::CoreFxError> {
        let clock = jobs.clock().clone();
        let authz_arc = Arc::new(authz.clone());

        let ledger = Arc::new(FxLedger::init(cala, journal_id, clock).await?);

        let chart_of_accounts_integrations = Arc::new(ChartOfAccountsIntegrations::new(
            authz_arc,
            Arc::new(internal_domain_configs.clone()),
        ));

        Ok(Self {
            ledger,
            chart_of_accounts_integrations,
        })
    }

    pub fn chart_of_accounts_integrations(&self) -> &ChartOfAccountsIntegrations<Perms> {
        &self.chart_of_accounts_integrations
    }

    pub fn ledger(&self) -> &FxLedger {
        &self.ledger
    }
}
