pub mod csv;

use super::entity::{Chart, NewChartAccountDetails};
use crate::primitives::{AccountSpec, CalaAccountSetId, CalaJournalId};

pub(super) struct BulkAccountImport<'a> {
    chart: &'a mut Chart,
    journal_id: CalaJournalId,
}

impl<'a> BulkAccountImport<'a> {
    pub fn new(chart: &'a mut Chart, journal_id: CalaJournalId) -> Self {
        Self { chart, journal_id }
    }

    pub(super) fn import(self, account_specs: Vec<AccountSpec>) -> BulkImportResult {
        let mut new_account_sets = Vec::new();
        let mut new_connections = Vec::new();
        //problems:
        // 1. parent is not created before child
        // 2. parent is created but it is a new entity and not persisted
        // Solution:
        // for 1.sort account_specs so that parents are created before children
        //
        for spec in account_specs {
            // for 2. persist each iteration or consider both entity and new entity
            // add find_new_mut to nested
            if let es_entity::Idempotent::Executed(NewChartAccountDetails {
                parent_account_set_id,
                new_account_set,
            }) = self
                .chart
                .create_node_without_verifying_parent(&spec, self.journal_id)
            {
                let account_set_id = new_account_set.id;
                new_account_sets.push(new_account_set);
                if let Some(parent) = parent_account_set_id {
                    new_connections.push((parent, account_set_id));
                }
            }
        }
        // let new_account_set_ids = new_account_sets.iter().map(|a| a.id).collect::<Vec<_>>();
        // if new_account_sets.is_empty() {
        //     // return Ok((chart, None));
        // }
        unimplemented!()
    }
}

pub(super) struct BulkImportResult {
    pub new_account_sets: Vec<cala_ledger::account_set::NewAccountSet>,
    pub new_account_set_ids: Vec<CalaAccountSetId>,
    pub new_connections: Vec<(CalaAccountSetId, CalaAccountSetId)>,
}
