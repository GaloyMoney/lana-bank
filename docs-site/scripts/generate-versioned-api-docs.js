#!/usr/bin/env node

/**
 * Generate API documentation for all versioned docs from versioned schemas.
 *
 * This script:
 * 1. Reads versions.json to get all doc versions
 * 2. For each version, temporarily swaps in the versioned schemas
 * 3. Runs the standard docusaurus graphql-to-doc commands
 * 4. Copies the generated docs to the appropriate versioned_docs folder
 * 5. Restores the original schemas
 * 6. Generates events docs for each version
 *
 * Schema files are expected at: schemas/versions/{version}/admin.graphql, customer.graphql, events.json
 */

const { execSync } = require("child_process");
const fs = require("fs");
const path = require("path");

const DOCS_SITE_DIR = path.join(__dirname, "..");
const VERSIONS_FILE = path.join(DOCS_SITE_DIR, "versions.json");
const SCHEMAS_DIR = path.join(DOCS_SITE_DIR, "schemas", "versions");
const VERSIONED_DOCS_DIR = path.join(DOCS_SITE_DIR, "versioned_docs");

// Original schema locations (as referenced in docusaurus.config.js)
const ADMIN_SCHEMA_PATH = path.join(DOCS_SITE_DIR, "..", "lana", "admin-server", "src", "graphql", "schema.graphql");
const CUSTOMER_SCHEMA_PATH = path.join(DOCS_SITE_DIR, "..", "lana", "customer-server", "src", "graphql", "schema.graphql");

// Generated docs location (where docusaurus outputs)
const GENERATED_ADMIN_DIR = path.join(DOCS_SITE_DIR, "docs", "api", "admin");
const GENERATED_CUSTOMER_DIR = path.join(DOCS_SITE_DIR, "docs", "api", "customer");

/**
 * Get all versions from versions.json
 */
function getVersions() {
  if (!fs.existsSync(VERSIONS_FILE)) {
    console.log("No versions.json found, skipping versioned docs generation.");
    return [];
  }
  return JSON.parse(fs.readFileSync(VERSIONS_FILE, "utf8"));
}

/**
 * Recursively copy a directory
 */
function copyDirSync(src, dest) {
  if (!fs.existsSync(src)) return 0;

  fs.mkdirSync(dest, { recursive: true });
  let count = 0;

  const entries = fs.readdirSync(src, { withFileTypes: true });
  for (const entry of entries) {
    const srcPath = path.join(src, entry.name);
    const destPath = path.join(dest, entry.name);

    if (entry.isDirectory()) {
      count += copyDirSync(srcPath, destPath);
    } else {
      fs.copyFileSync(srcPath, destPath);
      count++;
    }
  }
  return count;
}

/**
 * Recursively remove a directory
 */
function rmDirSync(dir) {
  if (!fs.existsSync(dir)) return;

  const entries = fs.readdirSync(dir, { withFileTypes: true });
  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      rmDirSync(fullPath);
    } else {
      fs.unlinkSync(fullPath);
    }
  }
  fs.rmdirSync(dir);
}

/**
 * Backup a file
 */
function backupFile(filePath) {
  const backupPath = filePath + ".backup";
  if (fs.existsSync(filePath)) {
    fs.copyFileSync(filePath, backupPath);
    return backupPath;
  }
  return null;
}

/**
 * Restore a file from backup
 */
function restoreFile(filePath) {
  const backupPath = filePath + ".backup";
  if (fs.existsSync(backupPath)) {
    fs.copyFileSync(backupPath, filePath);
    fs.unlinkSync(backupPath);
  }
}

/**
 * Generate events documentation for a specific version
 */
function generateEventsDocs(eventsSchemaPath, outputPath) {
  const DESCRIPTIONS_PATH = path.join(__dirname, "event-descriptions.json");

  if (!fs.existsSync(eventsSchemaPath)) {
    console.log(`  Events schema not found: ${eventsSchemaPath}`);
    return false;
  }

  const schema = JSON.parse(fs.readFileSync(eventsSchemaPath, "utf-8"));

  let descriptions = {};
  if (fs.existsSync(DESCRIPTIONS_PATH)) {
    descriptions = JSON.parse(fs.readFileSync(DESCRIPTIONS_PATH, "utf-8"));
  }

  const MODULE_ORDER = [
    { key: "Access", enumName: "CoreAccessEvent", prefix: "core.access" },
    { key: "Credit", enumName: "CoreCreditEvent", prefix: "core.credit" },
    { key: "Custody", enumName: "CoreCustodyEvent", prefix: "core.custody" },
    { key: "Customer", enumName: "CoreCustomerEvent", prefix: "core.customer" },
    { key: "Deposit", enumName: "CoreDepositEvent", prefix: "core.deposit" },
    { key: "Price", enumName: "CorePriceEvent", prefix: "core.price" },
    { key: "Report", enumName: "CoreReportEvent", prefix: "core.report" },
    { key: "Governance", enumName: "GovernanceEvent", prefix: "governance" },
  ];

  function extractEventsFromSchema(schema, enumName) {
    const definitions = schema.definitions || schema.$defs || {};
    let enumDef = definitions[enumName];

    if (!enumDef) {
      const moduleKey = enumName.replace(/^Core/, "").replace(/Event$/, "");
      const mainVariant = schema.oneOf?.find(
        (v) =>
          v.properties?.module?.const === moduleKey ||
          v.properties?.module?.const ===
            moduleKey.charAt(0).toUpperCase() + moduleKey.slice(1)
      );

      if (mainVariant && mainVariant.oneOf) {
        return extractInlinedEvents(mainVariant);
      }
      return [];
    }

    const events = [];
    const variants = enumDef.oneOf || enumDef.anyOf || [];

    for (const variant of variants) {
      const typeProperty = variant.properties?.type;
      if (!typeProperty) continue;

      const eventName =
        typeProperty.const || (typeProperty.enum && typeProperty.enum[0]);
      if (!eventName) continue;

      const fields = [];
      for (const [fieldName] of Object.entries(variant.properties || {})) {
        if (fieldName === "type") continue;
        fields.push(fieldName);
      }

      events.push({ name: eventName, fields });
    }

    return events;
  }

  function extractInlinedEvents(variant) {
    const events = [];
    for (const subVariant of variant.oneOf || []) {
      for (const [propName, propDef] of Object.entries(
        subVariant.properties || {}
      )) {
        if (propName === "module") continue;
        const fields = [];
        if (propDef.properties) {
          for (const fieldName of Object.keys(propDef.properties)) {
            fields.push(fieldName);
          }
        }
        events.push({ name: propName, fields });
      }
    }
    return events;
  }

  function generateEventsTable(events, eventDescriptions) {
    let md = "| Event | Description | Payload Fields |\n";
    md += "|-------|-------------|----------------|\n";

    for (const event of events) {
      const description =
        eventDescriptions[event.name] || "No description available";
      const fields =
        event.fields.length > 0 ? "`" + event.fields.join("`, `") + "`" : "-";
      md += `| \`${event.name}\` | ${description} | ${fields} |\n`;
    }

    return md;
  }

  function generateModuleMarkdown(moduleName, events, descs, subsections) {
    const moduleDesc =
      descs?.module_description || `Events related to ${moduleName.toLowerCase()}.`;
    const eventDescriptions = descs?.events || {};

    let md = `## ${moduleName} Events\n\n`;
    md += `${moduleDesc}\n\n`;

    if (subsections && Object.keys(subsections).length > 0) {
      for (const [subsectionName, eventNames] of Object.entries(subsections)) {
        const subsectionEvents = events.filter((e) =>
          eventNames.includes(e.name)
        );
        if (subsectionEvents.length === 0) continue;

        md += `### ${subsectionName}\n\n`;
        md += generateEventsTable(subsectionEvents, eventDescriptions);
        md += "\n";
      }

      const subsectionedEvents = new Set(Object.values(subsections).flat());
      const remainingEvents = events.filter(
        (e) => !subsectionedEvents.has(e.name)
      );
      if (remainingEvents.length > 0) {
        md += generateEventsTable(remainingEvents, eventDescriptions);
      }
    } else {
      md += generateEventsTable(events, eventDescriptions);
    }

    md += "\n---\n\n";
    return md;
  }

  const t = descriptions._translations || {};

  let md = `---
sidebar_position: 2
title: ${t.title || "Domain Events"}
description: ${t.description || "Public domain events published by Lana Bank"}
---

# ${t.title || "Domain Events"}

${t.intro || "Lana Bank publishes domain events via the transactional outbox pattern. These events can be consumed by external systems for integration, analytics, and audit purposes."}

${t.serialization_note || "All events are serialized as JSON and include metadata for tracing and ordering."}

---

## ${t.event_structure || "Event Structure"}

${t.event_structure_intro || "Each event is wrapped in an envelope with the following structure:"}

\`\`\`json
{
  "id": "uuid",
  "event_type": "core.credit.facility-activated",
  "payload": { ... },
  "recorded_at": "2024-01-15T10:30:00Z",
  "trace_id": "trace-uuid"
}
\`\`\`

---

`;

  for (const module of MODULE_ORDER) {
    const events = extractEventsFromSchema(schema, module.enumName);
    if (events.length === 0) continue;

    const moduleDescriptions = descriptions[module.enumName] || {};
    const subsections = moduleDescriptions.subsections || {};
    md += generateModuleMarkdown(
      module.key,
      events,
      moduleDescriptions,
      subsections
    );
  }

  md += `## ${t.event_types_reference || "Event Types Reference"}

${t.event_types_intro || "All event types follow the naming convention:"} \`core.<module>.<event-name>\`

| ${t.module || "Module"} | ${t.event_type_prefix || "Event Type Prefix"} |
|--------|-------------------|
`;

  for (const module of MODULE_ORDER) {
    md += `| ${module.key} | \`${module.prefix}.*\` |\n`;
  }

  md += `
---

## ${t.consuming_events || "Consuming Events"}

${t.consuming_intro || "Events are published via the transactional outbox and can be consumed through:"}

1. **${t.direct_polling || "Direct database polling"}** - ${t.direct_polling_desc || "Query the outbox table"}
2. **${t.event_streaming || "Event streaming"}** - ${t.event_streaming_desc || "Integration with message queues (implementation dependent)"}
3. **${t.etl_pipelines || "ETL pipelines"}** - ${t.etl_pipelines_desc || "Via Meltano extraction"}

${t.contact_note || "For integration details, contact the platform team."}
`;

  fs.mkdirSync(path.dirname(outputPath), { recursive: true });
  fs.writeFileSync(outputPath, md);
  return true;
}

async function main() {
  console.log("Generating API docs for versioned documentation...\n");

  const versions = getVersions();
  if (versions.length === 0) {
    console.log("No versions to process.");
    return;
  }

  for (const version of versions) {
    console.log(`\n=== Processing version ${version} ===\n`);

    const versionSchemaDir = path.join(SCHEMAS_DIR, version);
    const versionDocsDir = path.join(VERSIONED_DOCS_DIR, `version-${version}`);

    if (!fs.existsSync(versionSchemaDir)) {
      console.log(`  Schema directory not found: ${versionSchemaDir}`);
      console.log(`  Skipping version ${version}`);
      continue;
    }

    if (!fs.existsSync(versionDocsDir)) {
      console.log(`  Versioned docs directory not found: ${versionDocsDir}`);
      console.log(`  Skipping version ${version}`);
      continue;
    }

    const versionedAdminSchema = path.join(versionSchemaDir, "admin.graphql");
    const versionedCustomerSchema = path.join(versionSchemaDir, "customer.graphql");
    const versionedEventsSchema = path.join(versionSchemaDir, "events.json");

    // Backup original schemas
    console.log("  Backing up original schemas...");
    backupFile(ADMIN_SCHEMA_PATH);
    backupFile(CUSTOMER_SCHEMA_PATH);

    try {
      // Swap in versioned schemas
      if (fs.existsSync(versionedAdminSchema)) {
        fs.copyFileSync(versionedAdminSchema, ADMIN_SCHEMA_PATH);
      }
      if (fs.existsSync(versionedCustomerSchema)) {
        fs.copyFileSync(versionedCustomerSchema, CUSTOMER_SCHEMA_PATH);
      }

      // Clear existing generated docs to ensure fresh generation
      rmDirSync(GENERATED_ADMIN_DIR);
      rmDirSync(GENERATED_CUSTOMER_DIR);

      // Run docusaurus graphql-to-doc commands
      console.log("  Generating Admin API docs...");
      try {
        execSync("npm run generate-api-docs:admin", {
          cwd: DOCS_SITE_DIR,
          stdio: "pipe",
        });
      } catch (e) {
        console.log(`    Warning: Admin API generation had issues`);
      }

      console.log("  Generating Customer API docs...");
      try {
        execSync("npm run generate-api-docs:customer", {
          cwd: DOCS_SITE_DIR,
          stdio: "pipe",
        });
      } catch (e) {
        console.log(`    Warning: Customer API generation had issues`);
      }

      // Run fix-category-keys on generated docs
      console.log("  Adding category keys...");
      try {
        execSync("node scripts/fix-category-keys.js", {
          cwd: DOCS_SITE_DIR,
          stdio: "pipe",
        });
      } catch (e) {
        // Ignore errors, descriptions might not all be filled
      }

      // Copy generated docs to versioned docs
      console.log("  Copying to versioned docs...");
      const adminDestDir = path.join(versionDocsDir, "api", "admin");
      const customerDestDir = path.join(versionDocsDir, "api", "customer");

      // Remove existing versioned API docs first
      rmDirSync(adminDestDir);
      rmDirSync(customerDestDir);

      if (fs.existsSync(GENERATED_ADMIN_DIR)) {
        const count = copyDirSync(GENERATED_ADMIN_DIR, adminDestDir);
        console.log(`    Copied ${count} files to api/admin`);
      }

      if (fs.existsSync(GENERATED_CUSTOMER_DIR)) {
        const count = copyDirSync(GENERATED_CUSTOMER_DIR, customerDestDir);
        console.log(`    Copied ${count} files to api/customer`);
      }

      // Generate Events docs
      if (fs.existsSync(versionedEventsSchema)) {
        console.log("  Generating Events docs...");
        const eventsOutputPath = path.join(versionDocsDir, "api", "events.md");
        if (generateEventsDocs(versionedEventsSchema, eventsOutputPath)) {
          console.log("    Generated events.md");
        }
      }

    } finally {
      // Restore original schemas
      console.log("  Restoring original schemas...");
      restoreFile(ADMIN_SCHEMA_PATH);
      restoreFile(CUSTOMER_SCHEMA_PATH);
    }
  }

  // Regenerate current docs with original schemas (to ensure they're correct)
  console.log("\n=== Regenerating current docs ===\n");
  try {
    execSync("npm run generate-api-docs", {
      cwd: DOCS_SITE_DIR,
      stdio: "inherit",
    });
  } catch (e) {
    console.log("Warning: Current docs regeneration had issues");
  }

  console.log("\n=== Done generating versioned API docs ===\n");
}

main().catch(console.error);
