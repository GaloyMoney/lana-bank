use serde::Deserialize;
use sqlx::{PgPool, Row};
use uuid::Uuid;

// ── LANA side ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ChartRow {
    pub id: Uuid,
    pub name: String,
    pub reference: String,
    pub account_set_id: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct ChartNodeRow {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub parent_code: Option<String>,
    pub normal_balance_type: String,
    pub account_set_id: Uuid,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountSpec {
    pub parent: Option<AccountCode>,
    pub code: AccountCode,
    pub name: AccountName,
    pub normal_balance_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountCode {
    pub sections: Vec<AccountCodeSection>,
}

impl AccountCode {
    pub fn display(&self) -> String {
        self.sections
            .iter()
            .map(|s| s.code.as_str())
            .collect::<Vec<_>>()
            .join(".")
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountCodeSection {
    pub code: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountName {
    pub name: String,
}

pub async fn load_charts(pool: &PgPool) -> anyhow::Result<Vec<ChartRow>> {
    let rows = sqlx::query(
        r#"
        SELECT id, event
        FROM core_chart_events
        WHERE sequence = 1
          AND event_type = 'initialized'
        ORDER BY recorded_at
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut charts = Vec::new();
    for row in rows {
        let id: Uuid = row.get("id");
        let evt: serde_json::Value = row.get("event");

        let name = evt
            .get("name")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("(unknown)")
            .to_string();
        let reference = evt
            .get("reference")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
            .to_string();
        let account_set_id = evt
            .get("account_set_id")
            .and_then(serde_json::Value::as_str)
            .and_then(|s| s.parse::<Uuid>().ok());
        charts.push(ChartRow {
            id,
            name,
            reference,
            account_set_id,
        });
    }
    Ok(charts)
}

pub async fn load_chart_nodes(pool: &PgPool, chart_id: Uuid) -> anyhow::Result<Vec<ChartNodeRow>> {
    let rows = sqlx::query(
        r#"
        SELECT id, event
        FROM core_chart_node_events
        WHERE sequence = 1
          AND event_type = 'initialized'
          AND (event->>'chart_id')::uuid = $1
        ORDER BY id
        "#,
    )
    .bind(chart_id)
    .fetch_all(pool)
    .await?;

    let mut nodes = Vec::new();
    for row in rows {
        let id: Uuid = row.get("id");
        let evt: serde_json::Value = row.get("event");

        let spec: AccountSpec = match serde_json::from_value(evt["spec"].clone()) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let account_set_id: Uuid = match evt["ledger_account_set_id"]
            .as_str()
            .and_then(|s| s.parse().ok())
        {
            Some(id) => id,
            None => continue,
        };

        let code = spec.code.display();
        let parent_code = spec.parent.as_ref().map(|p| p.display());

        nodes.push(ChartNodeRow {
            id,
            code,
            name: spec.name.name,
            parent_code,
            normal_balance_type: spec.normal_balance_type,
            account_set_id,
        });
    }
    Ok(nodes)
}

// ── CALA side ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CalaAccountSetRow {
    pub id: Uuid,
    pub name: String,
    pub external_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CalaSetMemberSetRow {
    pub account_set_id: Uuid,
    pub member_account_set_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct CalaSetMemberAccountRow {
    pub account_set_id: Uuid,
    pub account_id: Uuid,
    pub account_code: String,
    pub account_name: String,
    pub account_external_id: Option<String>,
    pub normal_balance_type: String,
    pub transitive: bool,
}

pub async fn load_account_sets(pool: &PgPool) -> anyhow::Result<Vec<CalaAccountSetRow>> {
    let rows = sqlx::query(
        r#"
        SELECT id, name, external_id
        FROM cala_account_sets
        ORDER BY name
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| CalaAccountSetRow {
            id: r.get("id"),
            name: r.get("name"),
            external_id: r.get("external_id"),
        })
        .collect())
}

pub async fn load_set_member_sets(pool: &PgPool) -> anyhow::Result<Vec<CalaSetMemberSetRow>> {
    let rows = sqlx::query(
        r#"
        SELECT account_set_id, member_account_set_id
        FROM cala_account_set_member_account_sets
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| CalaSetMemberSetRow {
            account_set_id: r.get("account_set_id"),
            member_account_set_id: r.get("member_account_set_id"),
        })
        .collect())
}

pub async fn load_set_member_accounts(
    pool: &PgPool,
) -> anyhow::Result<Vec<CalaSetMemberAccountRow>> {
    let rows = sqlx::query(
        r#"
        SELECT
            m.account_set_id,
            m.member_account_id AS account_id,
            a.code AS account_code,
            a.name AS account_name,
            a.external_id AS account_external_id,
            a.normal_balance_type::text AS normal_balance_type,
            m.transitive
        FROM cala_account_set_member_accounts m
        JOIN cala_accounts a ON a.id = m.member_account_id
        ORDER BY a.code
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| CalaSetMemberAccountRow {
            account_set_id: r.get("account_set_id"),
            account_id: r.get("account_id"),
            account_code: r.get("account_code"),
            account_name: r.get("account_name"),
            account_external_id: r.get("account_external_id"),
            normal_balance_type: r.get("normal_balance_type"),
            transitive: r.get("transitive"),
        })
        .collect())
}

// ── Product integration configs ───────────────────────────────────

#[derive(Debug, Clone)]
pub struct ProductConfigRow {
    pub key: String,
    pub value: serde_json::Value,
}

pub async fn load_product_configs(pool: &PgPool) -> anyhow::Result<Vec<ProductConfigRow>> {
    let rows = sqlx::query(
        r#"
        SELECT DISTINCT ON (key) key, value
        FROM core_domain_config_events_rollup
        WHERE key IN (
            'credit-chart-of-accounts-integration',
            'deposit-chart-of-accounts-integration'
        )
        AND event_type = 'updated'
        AND value IS NOT NULL
        ORDER BY key, version DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| ProductConfigRow {
            key: r.get("key"),
            value: r.get("value"),
        })
        .collect())
}

// ── Balances ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AccountBalanceRow {
    pub account_id: Uuid,
    pub journal_id: Uuid,
    pub currency: String,
    pub settled_dr: String,
    pub settled_cr: String,
    pub pending_dr: String,
    pub pending_cr: String,
    pub encumbrance_dr: String,
    pub encumbrance_cr: String,
}

pub async fn load_balances(pool: &PgPool) -> anyhow::Result<Vec<AccountBalanceRow>> {
    let rows = sqlx::query(
        r#"
        SELECT
            account_id,
            journal_id,
            currency,
            latest_values
        FROM cala_current_balances
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let vals: serde_json::Value = r.get("latest_values");
            let extract = |layer: &str, side: &str| -> String {
                vals.get(layer)
                    .and_then(|l| l.get(side))
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("0")
                    .to_string()
            };
            AccountBalanceRow {
                account_id: r.get("account_id"),
                journal_id: r.get("journal_id"),
                currency: r.get("currency"),
                settled_dr: extract("settled", "dr_balance"),
                settled_cr: extract("settled", "cr_balance"),
                pending_dr: extract("pending", "dr_balance"),
                pending_cr: extract("pending", "cr_balance"),
                encumbrance_dr: extract("encumbrance", "dr_balance"),
                encumbrance_cr: extract("encumbrance", "cr_balance"),
            }
        })
        .collect())
}
