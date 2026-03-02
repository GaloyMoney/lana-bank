use std::path::Path;

use anyhow::Result;

use crate::{Violation, WorkspaceRule};

const RULE_NAME: &str = "graphql-id-naming";

/// Rule that enforces GraphQL ID naming conventions in schema.graphql files.
///
/// Three checks:
/// 1. Entity types with `id: ID!` must also have a `<camelCasedTypeName>Id: UUID!` field
/// 2. Input types must not have a bare `id: UUID!` field
/// 3. Query fields should not have redundant `<queryName>Id: UUID!` parameters
pub struct GraphqlIdNamingRule;

impl GraphqlIdNamingRule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GraphqlIdNamingRule {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
struct SchemaField {
    name: String,
    field_type: String,
    line_number: usize,
    params: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
struct SchemaBlock {
    kind: BlockKind,
    name: String,
    fields: Vec<SchemaField>,
    start_line: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum BlockKind {
    Type,
    Input,
}

fn parse_schema(content: &str) -> Vec<SchemaBlock> {
    let mut blocks = Vec::new();
    let mut current_block: Option<SchemaBlock> = None;
    let mut brace_depth = 0;

    for (line_idx, line) in content.lines().enumerate() {
        let line_number = line_idx + 1;
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if current_block.is_none() {
            if let Some(block) = try_parse_block_start(trimmed, line_number) {
                current_block = Some(block);
                brace_depth = 1;
                continue;
            }
        } else if let Some(ref mut block) = current_block {
            brace_depth += trimmed.matches('{').count();
            brace_depth -= trimmed.matches('}').count();

            if brace_depth == 0 {
                blocks.push(block.clone());
                current_block = None;
                continue;
            }

            // Only parse fields at depth 1 (direct children)
            if brace_depth == 1 {
                if let Some(field) = try_parse_field(trimmed, line_number) {
                    block.fields.push(field);
                }
            }
        }
    }

    blocks
}

fn try_parse_block_start(line: &str, line_number: usize) -> Option<SchemaBlock> {
    let (kind, rest) = if let Some(rest) = line.strip_prefix("type ") {
        (BlockKind::Type, rest)
    } else if let Some(rest) = line.strip_prefix("input ") {
        (BlockKind::Input, rest)
    } else {
        return None;
    };

    // Extract type name (before any implements/extends/{ etc)
    let name = rest
        .split_whitespace()
        .next()?
        .trim_end_matches('{')
        .to_string();

    if name.is_empty() {
        return None;
    }

    Some(SchemaBlock {
        kind,
        name,
        fields: Vec::new(),
        start_line: line_number,
    })
}

fn try_parse_field(line: &str, line_number: usize) -> Option<SchemaField> {
    let trimmed = line.trim();

    // Skip directives, comments, inline fragments
    if trimmed.starts_with('@')
        || trimmed.starts_with('#')
        || trimmed.starts_with("...")
        || trimmed == "}"
        || trimmed == "{"
    {
        return None;
    }

    // Parse field: name(params): Type or name: Type
    // Find the colon that separates field name/params from return type.
    // If there are parens, find the colon after the closing paren.
    let colon_pos = if let Some(paren_start) = trimmed.find('(') {
        let paren_end = trimmed[paren_start..].find(')').map(|p| paren_start + p)?;
        trimmed[paren_end..].find(':').map(|p| paren_end + p)?
    } else {
        trimmed.find(':')?
    };

    let name_and_params = &trimmed[..colon_pos];
    let field_type = trimmed[colon_pos + 1..]
        .trim()
        .trim_end_matches('!')
        .trim_start_matches('[')
        .trim_end_matches(']')
        .trim_end_matches('!')
        .trim()
        .to_string();

    let (name, params) = if let Some(paren_start) = name_and_params.find('(') {
        let name = name_and_params[..paren_start].trim().to_string();
        let params_str = &name_and_params[paren_start..];
        let params = parse_params(params_str);
        (name, params)
    } else {
        (name_and_params.trim().to_string(), Vec::new())
    };

    if name.is_empty() {
        return None;
    }

    Some(SchemaField {
        name,
        field_type,
        line_number,
        params,
    })
}

fn parse_params(params_str: &str) -> Vec<(String, String)> {
    let inner = params_str
        .trim_start_matches('(')
        .trim_end_matches(')')
        .trim();

    if inner.is_empty() {
        return Vec::new();
    }

    inner
        .split(',')
        .filter_map(|param| {
            let (name, typ) = param.trim().split_once(':')?;
            Some((
                name.trim().to_string(),
                typ.trim()
                    .trim_end_matches('!')
                    .trim_start_matches('[')
                    .trim_end_matches(']')
                    .trim_end_matches('!')
                    .trim()
                    .to_string(),
            ))
        })
        .collect()
}

fn type_name_to_camel_case_id(type_name: &str) -> String {
    let mut chars = type_name.chars();
    let first = chars.next().unwrap().to_lowercase().to_string();
    format!("{}{}Id", first, chars.collect::<String>())
}

fn is_excluded_type(name: &str) -> bool {
    name.ends_with("Payload")
        || name.ends_with("Connection")
        || name.ends_with("Edge")
        || name.ends_with("PageInfo")
}

/// Check 1: Entity types with `id: ID!` must have `<camelCasedTypeName>Id: UUID!`
fn check_entity_dual_id(block: &SchemaBlock, schema_file: &str) -> Vec<Violation> {
    if block.kind != BlockKind::Type || is_excluded_type(&block.name) {
        return Vec::new();
    }

    let has_id_field = block
        .fields
        .iter()
        .any(|f| f.name == "id" && f.field_type == "ID");

    if !has_id_field {
        return Vec::new();
    }

    let expected_field = type_name_to_camel_case_id(&block.name);

    let has_expected = block.fields.iter().any(|f| f.name == expected_field);

    if has_expected {
        return Vec::new();
    }

    vec![
        Violation::new(
            RULE_NAME,
            schema_file,
            format!(
                "Type `{}` has `id: ID!` but is missing `{}`",
                block.name, expected_field
            ),
        )
        .with_line(block.start_line),
    ]
}

/// Check 2: Input types must not have a bare `id: UUID!` field
fn check_input_no_bare_id(block: &SchemaBlock, schema_file: &str) -> Vec<Violation> {
    if block.kind != BlockKind::Input {
        return Vec::new();
    }

    block
        .fields
        .iter()
        .filter(|f| f.name == "id" && f.field_type == "UUID")
        .map(|f| {
            Violation::new(
                RULE_NAME,
                schema_file,
                format!(
                    "Input `{}` has bare `id: UUID!` field — use a descriptive name instead",
                    block.name
                ),
            )
            .with_line(f.line_number)
        })
        .collect()
}

/// Check 3: Query fields should not have redundant `<queryName>Id: UUID!` params
fn check_query_id_params(block: &SchemaBlock, schema_file: &str) -> Vec<Violation> {
    if block.kind != BlockKind::Type || block.name != "Query" {
        return Vec::new();
    }

    block
        .fields
        .iter()
        .filter_map(|f| {
            let redundant_param = format!("{}Id", f.name);
            let has_redundant = f
                .params
                .iter()
                .any(|(name, typ)| name == &redundant_param && typ == "UUID");
            if has_redundant {
                Some(
                    Violation::new(
                        RULE_NAME,
                        schema_file,
                        format!(
                            "Query `{}` has redundant param `{}: UUID!` — use `id: UUID!` instead",
                            f.name, redundant_param
                        ),
                    )
                    .with_line(f.line_number),
                )
            } else {
                None
            }
        })
        .collect()
}

impl WorkspaceRule for GraphqlIdNamingRule {
    fn name(&self) -> &'static str {
        RULE_NAME
    }

    fn description(&self) -> &'static str {
        "Enforce GraphQL ID naming conventions in schema files"
    }

    fn check_workspace(&self, workspace_root: &Path) -> Result<Vec<Violation>> {
        let schema_paths = [
            "lana/admin-server/src/graphql/schema.graphql",
            "lana/customer-server/src/graphql/schema.graphql",
        ];

        let mut violations = Vec::new();

        for schema_path in &schema_paths {
            let full_path = workspace_root.join(schema_path);
            if !full_path.exists() {
                continue;
            }

            let content = std::fs::read_to_string(&full_path)?;
            let blocks = parse_schema(&content);

            for block in &blocks {
                violations.extend(check_entity_dual_id(block, schema_path));
                violations.extend(check_input_no_bare_id(block, schema_path));
                violations.extend(check_query_id_params(block, schema_path));
            }
        }

        Ok(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_and_check(schema: &str) -> Vec<Violation> {
        let blocks = parse_schema(schema);
        let mut violations = Vec::new();
        for block in &blocks {
            violations.extend(check_entity_dual_id(block, "test.graphql"));
            violations.extend(check_input_no_bare_id(block, "test.graphql"));
            violations.extend(check_query_id_params(block, "test.graphql"));
        }
        violations
    }

    #[test]
    fn test_valid_entity_with_dual_id() {
        let schema = r#"
type Customer {
	id: ID!
	customerId: UUID!
	email: String!
}
"#;
        let violations = parse_and_check(schema);
        assert!(
            violations.is_empty(),
            "Expected no violations: {violations:?}"
        );
    }

    #[test]
    fn test_entity_missing_typed_id() {
        let schema = r#"
type Customer {
	id: ID!
	email: String!
}
"#;
        let violations = parse_and_check(schema);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("customerId"));
    }

    #[test]
    fn test_entity_wrong_id_name() {
        let schema = r#"
type CreditFacilityDisbursal {
	id: ID!
	disbursalId: UUID!
	amount: UsdCents!
}
"#;
        let violations = parse_and_check(schema);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("creditFacilityDisbursalId"));
    }

    #[test]
    fn test_payload_types_are_excluded() {
        let schema = r#"
type CustomerCreatePayload {
	id: ID!
	customer: Customer!
}

type CustomerConnection {
	id: ID!
	edges: [CustomerEdge!]!
}

type CustomerEdge {
	id: ID!
	node: Customer!
}
"#;
        let violations = parse_and_check(schema);
        assert!(
            violations.is_empty(),
            "Payload/Connection/Edge types should be excluded: {violations:?}"
        );
    }

    #[test]
    fn test_type_without_id_is_ignored() {
        let schema = r#"
type ChartNode {
	name: String!
	accountCode: String!
}
"#;
        let violations = parse_and_check(schema);
        assert!(
            violations.is_empty(),
            "Types without id: ID! should be ignored: {violations:?}"
        );
    }

    #[test]
    fn test_input_no_bare_id() {
        let schema = r#"
input CustomerUpdateInput {
	id: UUID!
	email: String!
}
"#;
        let violations = parse_and_check(schema);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("bare `id: UUID!`"));
    }

    #[test]
    fn test_input_with_named_id_is_ok() {
        let schema = r#"
input CustomerUpdateInput {
	customerId: UUID!
	email: String!
}
"#;
        let violations = parse_and_check(schema);
        assert!(
            violations.is_empty(),
            "Named ID fields in inputs are fine: {violations:?}"
        );
    }

    #[test]
    fn test_query_redundant_param() {
        let schema = r#"
type Query {
	customer(customerId: UUID!): Customer
	withdrawal(id: UUID!): Withdrawal
}
"#;
        let violations = parse_and_check(schema);
        assert_eq!(violations.len(), 1);
        assert!(
            violations[0]
                .message
                .contains("redundant param `customerId: UUID!`")
        );
    }

    #[test]
    fn test_query_with_id_param_is_ok() {
        let schema = r#"
type Query {
	customer(id: UUID!): Customer
	withdrawal(id: UUID!): Withdrawal
}
"#;
        let violations = parse_and_check(schema);
        assert!(
            violations.is_empty(),
            "Query with id: UUID! param is fine: {violations:?}"
        );
    }

    #[test]
    fn test_multi_word_type_name_camel_case() {
        assert_eq!(
            type_name_to_camel_case_id("CreditFacilityDisbursal"),
            "creditFacilityDisbursalId"
        );
        assert_eq!(type_name_to_camel_case_id("Customer"), "customerId");
        assert_eq!(
            type_name_to_camel_case_id("ChartOfAccounts"),
            "chartOfAccountsId"
        );
    }
}
