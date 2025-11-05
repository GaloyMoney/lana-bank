use sqlx::PgPool;

use es_entity::*;

use crate::primitives::{ChartId, ChartNodeId};

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
}
impl ChartNodeRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }
}

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "Chart",
    err = "ChartOfAccountsError",
    columns(reference(ty = "String")),
    tbl_prefix = "core"
)]
pub struct ChartRepo {
    pool: PgPool,

    #[es_repo(nested)]
    chart_nodes: ChartNodeRepo,
}

impl ChartRepo {
    pub fn new(pool: &PgPool) -> Self {
        let chart_nodes = ChartNodeRepo::new(pool);
        Self {
            pool: pool.clone(),
            chart_nodes,
        }
    }
}
