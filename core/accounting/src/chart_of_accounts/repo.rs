use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;

use crate::primitives::ChartId;
use crate::primitives::ChartNodeId;

use super::chart_node::*;
use super::{entity::*, error::ChartOfAccountsError};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "ChartNode",
    err = "ChartOfAccountsError",
    columns(chart_id(ty = "ChartId", update(persist = false), parent)),
    tbl_prefix = "core"
)]
struct ChartNodeRepo {
    #[allow(dead_code)]
    pool: PgPool,
    #[allow(dead_code)]
    clock: ClockHandle,
}
impl ChartNodeRepo {
    pub(crate) fn new(pool: &PgPool, clock: ClockHandle) -> Self {
        Self {
            pool: pool.clone(),
            clock,
        }
    }
}

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "Chart",
    err = "ChartOfAccountsError",
    columns(reference(ty = "String")),
    tbl_prefix = "core"
)]
pub(crate) struct ChartRepo {
    pool: PgPool,
    clock: ClockHandle,

    #[es_repo(nested)]
    chart_nodes: ChartNodeRepo,
}

impl ChartRepo {
    pub(crate) fn new(pool: &PgPool, clock: ClockHandle) -> Self {
        let chart_nodes = ChartNodeRepo::new(pool, clock.clone());
        Self {
            pool: pool.clone(),
            clock,
            chart_nodes,
        }
    }
}
