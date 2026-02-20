#!/usr/bin/env node

/**
 * CI validation script for API descriptions.
 *
 * Validates:
 * 1. All current operations have descriptions in both English and Spanish
 * 2. No descriptions are still using the default placeholder
 *
 * Note: Extra descriptions for operations that no longer exist in the current
 * schema are allowed (warnings only) since they may be needed for versioned docs.
 *
 * Operations are extracted directly from the GraphQL schema files,
 * so this script is independent of the doc generation pipeline.
 *
 * Exit codes:
 * 0 - All validations passed
 * 1 - Validation errors found
 */

const fs = require("fs");
const path = require("path");

const SCRIPTS_DIR = __dirname;
const ROOT_DIR = path.join(__dirname, "..");
const DESCRIPTIONS_FILE = path.join(SCRIPTS_DIR, "api-descriptions.json");
const DESCRIPTIONS_ES_FILE = path.join(SCRIPTS_DIR, "api-descriptions.es.json");

const SCHEMA_FILES = {
  admin: path.join(ROOT_DIR, "..", "lana", "admin-server", "src", "graphql", "schema.graphql"),
  customer: path.join(ROOT_DIR, "..", "lana", "customer-server", "src", "graphql", "schema.graphql"),
};

/**
 * Parse a GraphQL schema file and extract field names from a root type (Query or Mutation).
 */
function extractFieldNames(schemaContent, typeName) {
  const fields = new Set();
  // Match `type Query {` or `type Mutation {` and extract the block
  const regex = new RegExp(`^type\\s+${typeName}\\s*\\{`, "m");
  const match = regex.exec(schemaContent);
  if (!match) return fields;

  let depth = 1;
  let pos = match.index + match[0].length;
  let block = "";

  while (pos < schemaContent.length && depth > 0) {
    const ch = schemaContent[pos];
    if (ch === "{") depth++;
    else if (ch === "}") depth--;
    if (depth > 0) block += ch;
    pos++;
  }

  // Each field line looks like: `  fieldName(args): ReturnType`
  for (const line of block.split("\n")) {
    const fieldMatch = line.match(/^\s+(\w+)\s*[:(]/);
    if (fieldMatch) {
      fields.add(fieldMatch[1]);
    }
  }

  return fields;
}

/**
 * Get all current operations from the GraphQL schema files
 */
function getCurrentOperations() {
  const operations = {
    admin: { queries: new Set(), mutations: new Set() },
    customer: { queries: new Set(), mutations: new Set() },
  };

  for (const apiId of ["admin", "customer"]) {
    const schemaPath = SCHEMA_FILES[apiId];
    if (!fs.existsSync(schemaPath)) {
      console.error(`Schema file not found: ${schemaPath}`);
      process.exit(1);
    }

    const schema = fs.readFileSync(schemaPath, "utf8");
    operations[apiId].queries = extractFieldNames(schema, "Query");
    operations[apiId].mutations = extractFieldNames(schema, "Mutation");
  }

  return operations;
}

/**
 * Load descriptions from JSON file
 */
function loadDescriptions(filePath) {
  if (!fs.existsSync(filePath)) {
    return null;
  }
  return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

/**
 * Validate descriptions
 */
function validateDescriptions() {
  const errors = [];
  const warnings = [];

  // Load descriptions
  const enDescriptions = loadDescriptions(DESCRIPTIONS_FILE);
  const esDescriptions = loadDescriptions(DESCRIPTIONS_ES_FILE);

  if (!enDescriptions) {
    errors.push("English descriptions file not found: api-descriptions.json");
    return { errors, warnings };
  }

  if (!esDescriptions) {
    errors.push("Spanish descriptions file not found: api-descriptions.es.json");
    return { errors, warnings };
  }

  // Get current operations from GraphQL schemas
  const currentOperations = getCurrentOperations();

  const enDefault = enDescriptions._meta?.defaultDescription || "TODO: Add description";
  const esDefault = esDescriptions._meta?.defaultDescription || "TODO: Agregar descripcion";

  // Check each API
  for (const apiId of ["admin", "customer"]) {
    for (const typeKey of ["queries", "mutations"]) {
      const currentOps = currentOperations[apiId][typeKey];
      const enOps = enDescriptions[apiId]?.[typeKey] || {};
      const esOps = esDescriptions[apiId]?.[typeKey] || {};

      // Check for stale English descriptions (operations that no longer exist in current schema)
      // These are warnings, not errors, since they may be needed for versioned docs
      for (const opName of Object.keys(enOps)) {
        if (!currentOps.has(opName)) {
          warnings.push(`Extra English description: ${apiId}.${typeKey}.${opName} (not in current schema, may be for versioned docs)`);
        }
      }

      // Check for stale Spanish descriptions
      for (const opName of Object.keys(esOps)) {
        if (!currentOps.has(opName)) {
          warnings.push(`Extra Spanish description: ${apiId}.${typeKey}.${opName} (not in current schema, may be for versioned docs)`);
        }
      }

      // Check each current operation has proper descriptions
      for (const opName of currentOps) {
        // Check English description
        const enDesc = enOps[opName];
        if (!enDesc) {
          errors.push(`Missing English description: ${apiId}.${typeKey}.${opName}`);
        } else if (enDesc === enDefault) {
          errors.push(`Default English description not replaced: ${apiId}.${typeKey}.${opName}`);
        }

        // Check Spanish description
        const esDesc = esOps[opName];
        if (!esDesc) {
          errors.push(`Missing Spanish description: ${apiId}.${typeKey}.${opName}`);
        } else if (esDesc === esDefault) {
          errors.push(`Default Spanish description not replaced: ${apiId}.${typeKey}.${opName}`);
        }
      }
    }
  }

  return { errors, warnings };
}

function main() {
  console.log("Validating API descriptions...\n");

  const { errors, warnings } = validateDescriptions();

  // Print warnings
  if (warnings.length > 0) {
    console.log("Warnings:");
    for (const warning of warnings) {
      console.log(`  ⚠️  ${warning}`);
    }
    console.log("");
  }

  // Print errors
  if (errors.length > 0) {
    console.log("Errors:");
    for (const error of errors) {
      console.log(`  ❌ ${error}`);
    }
    console.log(`\n${errors.length} error(s) found.`);
    console.log("\nTo fix:");
    console.log("  1. For missing descriptions: Add descriptions to both api-descriptions.json and api-descriptions.es.json");
    console.log("  2. For default descriptions: Replace placeholder text with actual descriptions");
    console.log("  3. Run 'npm run generate-api-docs' to regenerate documentation");
    process.exit(1);
  }

  console.log("✅ All API descriptions are valid!");
  process.exit(0);
}

main();
