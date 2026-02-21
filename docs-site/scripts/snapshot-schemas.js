#!/usr/bin/env node

/**
 * Snapshot current GraphQL schemas for a documentation version.
 *
 * This script should be run AFTER `pnpm run docusaurus docs:version X.X.X`
 * to capture the schemas that correspond to that version.
 *
 * Usage:
 *   node scripts/snapshot-schemas.js 0.0.2
 *   pnpm run snapshot-schemas -- 0.0.2
 *
 * This copies:
 *   - ../lana/admin-server/src/graphql/schema.graphql → schemas/versions/{version}/admin.graphql
 *   - ../lana/customer-server/src/graphql/schema.graphql → schemas/versions/{version}/customer.graphql
 *   - schemas/lana_events.json → schemas/versions/{version}/events.json
 */

const fs = require("fs");
const path = require("path");

const DOCS_SITE_DIR = path.join(__dirname, "..");

// Source schema locations
const ADMIN_SCHEMA_SRC = path.join(
  DOCS_SITE_DIR,
  "..",
  "lana",
  "admin-server",
  "src",
  "graphql",
  "schema.graphql"
);
const CUSTOMER_SCHEMA_SRC = path.join(
  DOCS_SITE_DIR,
  "..",
  "lana",
  "customer-server",
  "src",
  "graphql",
  "schema.graphql"
);
const EVENTS_SCHEMA_SRC = path.join(
  DOCS_SITE_DIR,
  "schemas",
  "lana_events.json"
);

// Destination directory
const SCHEMAS_VERSIONS_DIR = path.join(DOCS_SITE_DIR, "schemas", "versions");

function main() {
  const version = process.argv[2];

  if (!version) {
    console.error("Usage: node scripts/snapshot-schemas.js <version>");
    console.error("Example: node scripts/snapshot-schemas.js 0.0.2");
    process.exit(1);
  }

  // Validate version format (basic check)
  if (!/^\d+\.\d+\.\d+(-[\w.]+)?$/.test(version)) {
    console.error(`Invalid version format: ${version}`);
    console.error("Expected format: X.Y.Z or X.Y.Z-suffix");
    process.exit(1);
  }

  const versionDir = path.join(SCHEMAS_VERSIONS_DIR, version);

  // Check if version already exists
  if (fs.existsSync(versionDir)) {
    console.error(`Schema snapshot already exists: ${versionDir}`);
    console.error("To re-snapshot, first delete the existing directory.");
    process.exit(1);
  }

  // Verify source files exist
  const sources = [
    { path: ADMIN_SCHEMA_SRC, name: "Admin schema" },
    { path: CUSTOMER_SCHEMA_SRC, name: "Customer schema" },
    { path: EVENTS_SCHEMA_SRC, name: "Events schema" },
  ];

  for (const source of sources) {
    if (!fs.existsSync(source.path)) {
      console.error(`${source.name} not found: ${source.path}`);
      process.exit(1);
    }
  }

  // Create version directory
  fs.mkdirSync(versionDir, { recursive: true });
  console.log(`Created: ${versionDir}`);

  // Copy schemas
  const copies = [
    { src: ADMIN_SCHEMA_SRC, dest: "admin.graphql" },
    { src: CUSTOMER_SCHEMA_SRC, dest: "customer.graphql" },
    { src: EVENTS_SCHEMA_SRC, dest: "events.json" },
  ];

  for (const { src, dest } of copies) {
    const destPath = path.join(versionDir, dest);
    fs.copyFileSync(src, destPath);
    console.log(`Copied: ${dest}`);
  }

  console.log(`\nSchema snapshot complete for version ${version}`);
  console.log("\nNext steps:");
  console.log("  1. Commit the schema snapshot: git add schemas/versions/");
  console.log("  2. Run build to generate versioned API docs: pnpm run build");
}

main();
