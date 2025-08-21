use anyhow::{Context, Result};
use clap::Parser;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use tokio_postgres::{Client, NoTls};
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "cala-dag-visualizer")]
#[command(about = "Generate a DOT file visualizing the Cala account DAG structure")]
struct Args {
    /// PostgreSQL connection string (e.g., "postgresql://user:password@localhost:5432/dbname")
    /// Can also be set via PG_CON environment variable
    #[arg(short, long)]
    connection: Option<String>,

    /// Output DOT file path
    #[arg(short, long, default_value = "cala_dag.dot")]
    output: String,
}

#[derive(Debug, Clone)]
struct Account {
    id: Uuid,
    name: String,
    is_account_set: bool,
}

#[derive(Debug, Clone)]
struct Membership {
    parent_id: Uuid,
    child_id: Uuid,
    is_transitive: bool,
    relationship_type: RelationshipType,
}

#[derive(Debug, Clone, PartialEq)]
enum RelationshipType {
    AccountToAccount, // account_set -> individual_account
    SetToSet,         // account_set -> account_set
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Get connection string from args or environment
    let connection_string = args.connection
        .or_else(|| std::env::var("PG_CON").ok())
        .context("PostgreSQL connection string must be provided via --connection or PG_CON environment variable")?;

    // Connect to PostgreSQL
    let (client, connection) = tokio_postgres::connect(&connection_string, NoTls)
        .await
        .context("Failed to connect to PostgreSQL")?;

    // Spawn the connection
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    println!("🔌 Connected to PostgreSQL database");

    // Query accounts and account sets
    let accounts = fetch_accounts(&client).await?;
    println!(
        "📊 Found {} total accounts ({} account sets, {} individual accounts)",
        accounts.len(),
        accounts.values().filter(|a| a.is_account_set).count(),
        accounts.values().filter(|a| !a.is_account_set).count()
    );

    // Query membership relationships
    let memberships = fetch_memberships(&client).await?;
    println!("🔗 Found {} membership relationships", memberships.len());

    // Show accounts that have names, plus parent sets needed for relationships
    let named_account_ids: HashSet<Uuid> = accounts
        .iter()
        .filter(|(_, account)| !account.name.trim().is_empty())
        .map(|(id, _)| *id)
        .collect();

    let mut included_ids = named_account_ids.clone();

    // Include parent account sets that connect to named accounts
    for membership in &memberships {
        if named_account_ids.contains(&membership.child_id) {
            included_ids.insert(membership.parent_id);
        }
    }

    let filtered_accounts: HashMap<Uuid, Account> = accounts
        .into_iter()
        .filter(|(id, _)| included_ids.contains(id))
        .collect();

    println!("📋 Showing {} accounts", filtered_accounts.len());

    // Generate DOT file
    generate_dot_file(&filtered_accounts, &memberships, &args.output)?;
    println!("✅ Generated DOT file: {}", args.output);
    println!("💡 To visualize: fdp -Tsvg {} -o cala_dag.svg", args.output);

    Ok(())
}

async fn fetch_accounts(client: &Client) -> Result<HashMap<Uuid, Account>> {
    let query = r#"
        SELECT 
            a.id,
            a.name,
            CASE WHEN s.id IS NOT NULL THEN true ELSE false END as is_account_set
        FROM cala_accounts a
        LEFT JOIN cala_account_sets s ON a.id = s.id
        ORDER BY a.name
    "#;

    let rows = client.query(query, &[]).await?;
    let mut accounts = HashMap::new();

    for row in rows {
        let id: Uuid = row.get(0);
        let account = Account {
            id,
            name: row.get(1),
            is_account_set: row.get(2),
        };
        accounts.insert(id, account);
    }

    Ok(accounts)
}

async fn fetch_memberships(client: &Client) -> Result<Vec<Membership>> {
    let mut memberships = Vec::new();

    // Fetch account set -> individual account memberships
    let account_memberships_query = r#"
        SELECT 
            m.account_set_id,
            m.member_account_id,
            m.transitive
        FROM cala_account_set_member_accounts m
    "#;

    let rows = client.query(account_memberships_query, &[]).await?;
    for row in rows {
        memberships.push(Membership {
            parent_id: row.get(0),
            child_id: row.get(1),
            is_transitive: row.get(2),
            relationship_type: RelationshipType::AccountToAccount,
        });
    }

    // Fetch account set -> account set memberships
    let set_memberships_query = r#"
        SELECT 
            m.account_set_id,
            m.member_account_set_id
        FROM cala_account_set_member_account_sets m
    "#;

    let rows = client.query(set_memberships_query, &[]).await?;
    for row in rows {
        memberships.push(Membership {
            parent_id: row.get(0),
            child_id: row.get(1),
            is_transitive: false, // Set-to-set relationships don't have transitive field
            relationship_type: RelationshipType::SetToSet,
        });
    }

    Ok(memberships)
}

fn generate_dot_file(
    accounts: &HashMap<Uuid, Account>,
    memberships: &Vec<Membership>,
    output_path: &str,
) -> Result<()> {
    let mut file = File::create(output_path)?;

    // Start DOT file
    writeln!(file, "digraph cala_dag {{")?;
    writeln!(file, "    rankdir=TB;")?;
    writeln!(file, "    node [fontsize=10, style=filled];")?;
    writeln!(file, "    edge [fontsize=8];")?;
    writeln!(file)?;

    // Add all nodes
    for account in accounts.values() {
        let node_id = format_uuid_short(account.id);
        let label = if account.name.trim().is_empty() {
            // Use the last 8 characters of UUID for better uniqueness
            let uuid_str = account.id.to_string();
            let unique_part = &uuid_str[uuid_str.len() - 8..];
            format!("{}...{}", &uuid_str[..8], unique_part)
        } else {
            account.name.replace("\"", "\\\"")
        };

        let (color, shape) = if account.is_account_set {
            ("lightblue", "box")
        } else {
            ("lightgreen", "ellipse")
        };

        writeln!(
            file,
            "    {} [label=\"{}\", fillcolor={}, shape={}];",
            node_id, label, color, shape
        )?;
    }

    writeln!(file)?;

    // Add edges
    for membership in memberships {
        if accounts.contains_key(&membership.parent_id)
            && accounts.contains_key(&membership.child_id)
        {
            let parent_id = format_uuid_short(membership.parent_id);
            let child_id = format_uuid_short(membership.child_id);

            let (color, style) = match (&membership.relationship_type, membership.is_transitive) {
                (RelationshipType::AccountToAccount, false) => ("blue", "solid"),
                (RelationshipType::AccountToAccount, true) => ("blue", "dashed"),
                (RelationshipType::SetToSet, _) => ("red", "solid"),
            };

            writeln!(
                file,
                "    {} -> {} [color={}, style={}];",
                parent_id, child_id, color, style
            )?;
        }
    }

    writeln!(file, "}}")?;
    Ok(())
}

fn format_uuid_short(uuid: Uuid) -> String {
    // Use the full UUID but replace hyphens with underscores for DOT compatibility
    let full = uuid.to_string().replace("-", "_");
    format!("n_{}", full)
}
