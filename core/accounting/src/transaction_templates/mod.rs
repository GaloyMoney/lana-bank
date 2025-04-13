use audit::AuditSvc;
use authz::PermissionCheck;

use cala_ledger::{CalaLedger, tx_template::TxTemplate};

use crate::primitives::{CoreAccountingAction, CoreAccountingObject, TransactionTemplateId};

#[derive(Clone)]
pub struct TransactionTemplates<Perms>
where
    Perms: PermissionCheck,
{
    _authz: Perms,
    cala: CalaLedger,
}

impl<Perms> TransactionTemplates<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
{
    pub fn new(authz: &Perms, cala: &CalaLedger) -> Self {
        Self {
            _authz: authz.clone(),
            cala: cala.clone(),
        }
    }

    pub async fn list(&self) -> Result<Vec<TransactionTemplate>, ()> {
        let templates = self
            .cala
            .tx_templates()
            .list(Default::default(), Default::default())
            .await
            .unwrap()
            .entities
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(templates)
    }
}

pub struct TransactionTemplate {
    pub id: TransactionTemplateId,
    pub code: String,
}

impl From<TxTemplate> for TransactionTemplate {
    fn from(value: TxTemplate) -> Self {
        todo!()
    }
}
