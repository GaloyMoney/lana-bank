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
 * Exit codes:
 * 0 - All validations passed
 * 1 - Validation errors found
 */

const fs = require("fs");
const path = require("path");

const SCRIPTS_DIR = __dirname;
const DOCS_DIR = path.join(__dirname, "..", "docs");
const DESCRIPTIONS_FILE = path.join(SCRIPTS_DIR, "api-descriptions.json");
const DESCRIPTIONS_ES_FILE = path.join(SCRIPTS_DIR, "api-descriptions.es.json");

/**
 * Convert kebab-case filename to camelCase operation name
 */
function filenameToCamelCase(filename) {
  const name = filename.replace(/\.mdx$/, "");
  return name.replace(/-([a-z])/g, (_, letter) => letter.toUpperCase());
}

/**
 * Find all operation files in an API directory
 */
function findOperationFiles(apiDir) {
  const operationsDir = path.join(DOCS_DIR, apiDir, "operations");
  const files = [];

  if (!fs.existsSync(operationsDir)) return files;

  const subdirs = ["queries", "mutations"];
  for (const subdir of subdirs) {
    const dir = path.join(operationsDir, subdir);
    if (!fs.existsSync(dir)) continue;

    const entries = fs.readdirSync(dir, { withFileTypes: true });
    for (const entry of entries) {
      if (entry.isFile() && entry.name.endsWith(".mdx")) {
        files.push({
          name: filenameToCamelCase(entry.name),
          type: subdir === "queries" ? "queries" : "mutations",
        });
      }
    }
  }

  return files;
}

/**
 * Get all current operations from the docs
 */
function getCurrentOperations() {
  const operations = {
    admin: { queries: new Set(), mutations: new Set() },
    customer: { queries: new Set(), mutations: new Set() },
  };

  for (const apiId of ["admin", "customer"]) {
    const files = findOperationFiles(`api/${apiId}`);
    for (const file of files) {
      operations[apiId][file.type].add(file.name);
    }
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

  // Get current operations from docs
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
