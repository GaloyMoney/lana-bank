pub mod error;

use std::collections::HashMap;
use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::{
    CalaLedger,
    tx_template::{TxTemplate, TxTemplatesByCodeCursor},
};

use crate::primitives::{CoreAccountingAction, CoreAccountingObject, TransactionTemplateId};

use error::TransactionTemplateError;

pub type TransactionTemplateCursor = TxTemplatesByCodeCursor;
pub type TransactionTemplate = TxTemplate;

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

    #[instrument(name = "core_accounting.transaction_template.list", skip(self), err)]
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
            entities: cursor.entities,
            has_next_page: cursor.has_next_page,
            end_cursor: cursor.end_cursor,
        })
    }

    #[instrument(
        name = "core_accounting.transaction_template.find_all",
        skip(self),
        err
    )]
    pub async fn find_all<T: From<TransactionTemplate>>(
        &self,
        ids: &[TransactionTemplateId],
    ) -> Result<HashMap<TransactionTemplateId, T>, TransactionTemplateError> {
        Ok(self.cala.tx_templates().find_all(ids).await?)
    }
}
