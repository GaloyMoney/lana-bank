pub mod error;

use audit::AuditSvc;
use authz::PermissionCheck;

use cala_ledger::{
    CalaLedger,
    tx_template::{TxTemplate, TxTemplatesByCodeCursor},
};
use error::TransactionTemplateError;

use crate::primitives::{CoreAccountingAction, CoreAccountingObject, TransactionTemplateId};

pub type TransactionTemplateCursor = TxTemplatesByCodeCursor;

#[derive(Clone)]
pub struct TransactionTemplates<Perms>
where
    Perms: PermissionCheck,
{
    authz: Perms,
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
            authz: authz.clone(),
            cala: cala.clone(),
        }
    }

    pub async fn list(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        args: es_entity::PaginatedQueryArgs<TransactionTemplateCursor>,
    ) -> Result<
        es_entity::PaginatedQueryRet<TransactionTemplate, TransactionTemplateCursor>,
        TransactionTemplateError,
    > {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_transaction_templates(),
                CoreAccountingAction::TRANSACTION_TEMPLATE_LIST,
            )
            .await?;

        let cursor = self
            .cala
            .tx_templates()
            .list(args, es_entity::ListDirection::Ascending)
            .await?;

        Ok(es_entity::PaginatedQueryRet {
            entities: cursor.entities.into_iter().map(Into::into).collect(),
            has_next_page: cursor.has_next_page,
            end_cursor: cursor.end_cursor,
        })
    }
}

pub struct TransactionTemplate {
    pub id: TransactionTemplateId,
    pub code: String,
}

impl From<TxTemplate> for TransactionTemplate {
    fn from(template: TxTemplate) -> Self {
        let id = template.id;
        let values = template.into_values();
        Self {
            id,
            code: values.code,
        }
    }
}

impl From<&TransactionTemplate> for TransactionTemplateCursor {
    fn from(template: &TransactionTemplate) -> Self {
        Self {
            id: template.id,
            code: template.code.clone(),
        }
    }
}
