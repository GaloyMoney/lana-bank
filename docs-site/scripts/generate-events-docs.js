#!/usr/bin/env node
/**
 * Generate Domain Events Documentation
 *
 * This script reads the JSON schema generated from Rust code and combines it
 * with human-readable descriptions to generate markdown documentation.
 *
 * Usage: node scripts/generate-events-docs.js [--locale en|es]
 */

const fs = require('fs');
const path = require('path');

const SCHEMA_PATH = path.join(__dirname, '../schemas/lana_events.json');
const DESCRIPTIONS_EN_PATH = path.join(__dirname, 'event-descriptions.json');
const DESCRIPTIONS_ES_PATH = path.join(__dirname, 'event-descriptions.es.json');
const OUTPUT_EN_PATH = path.join(__dirname, '../docs/api/events.md');
const OUTPUT_ES_PATH = path.join(__dirname, '../i18n/es/docusaurus-plugin-content-docs/current/api/events.md');

// Module display order and metadata
const MODULE_ORDER = [
  { key: 'Access', enumName: 'CoreAccessEvent', prefix: 'core.access' },
  { key: 'Credit', enumName: 'CoreCreditEvent', prefix: 'core.credit' },
  { key: 'Custody', enumName: 'CoreCustodyEvent', prefix: 'core.custody' },
  { key: 'Customer', enumName: 'CoreCustomerEvent', prefix: 'core.customer' },
  { key: 'Deposit', enumName: 'CoreDepositEvent', prefix: 'core.deposit' },
  { key: 'Price', enumName: 'CorePriceEvent', prefix: 'core.price' },
  { key: 'Report', enumName: 'CoreReportEvent', prefix: 'core.report' },
  { key: 'Governance', enumName: 'GovernanceEvent', prefix: 'governance' },
];

/**
 * Extract event variants and their fields from a JSON schema definition
 */
function extractEventsFromSchema(schema, enumName) {
  const definitions = schema.definitions || schema.$defs || {};

  // First, try to find in definitions (most events)
  let enumDef = definitions[enumName];

  // If not in definitions, check if it's inlined in the main schema (e.g., CorePriceEvent)
  if (!enumDef) {
    // Find the module variant in the main schema's oneOf
    const moduleKey = enumName.replace(/^Core/, '').replace(/Event$/, '');
    const mainVariant = schema.oneOf?.find(v =>
      v.properties?.module?.const === moduleKey ||
      v.properties?.module?.const === moduleKey.charAt(0).toUpperCase() + moduleKey.slice(1)
    );

    if (mainVariant && mainVariant.oneOf) {
      // This is an inlined event (like CorePriceEvent with untagged variant)
      return extractInlinedEvents(mainVariant);
    }

    console.warn(`Warning: Could not find definition for ${enumName}`);
    return [];
  }

  const events = [];
  const variants = enumDef.oneOf || enumDef.anyOf || [];

  for (const variant of variants) {
    // Get the "type" property which contains the event name (from serde tag)
    const typeProperty = variant.properties?.type;
    if (!typeProperty) continue;

    const eventName = typeProperty.const || (typeProperty.enum && typeProperty.enum[0]);
    if (!eventName) continue;

    // Get all other properties (the payload fields)
    const fields = [];
    for (const [fieldName, fieldDef] of Object.entries(variant.properties || {})) {
      if (fieldName === 'type') continue; // Skip the discriminator
      fields.push(fieldName);
    }

    events.push({
      name: eventName,
      fields: fields,
    });
  }

  return events;
}

/**
 * Extract events from inlined schema (for events without serde tag, like CorePriceEvent)
 */
function extractInlinedEvents(variant) {
  const events = [];

  for (const subVariant of (variant.oneOf || [])) {
    // Each subVariant has a property named after the event variant
    for (const [propName, propDef] of Object.entries(subVariant.properties || {})) {
      if (propName === 'module') continue;

      // The event name is the property name
      const fields = [];
      if (propDef.properties) {
        for (const fieldName of Object.keys(propDef.properties)) {
          fields.push(fieldName);
        }
      }

      events.push({
        name: propName,
        fields: fields,
      });
    }
  }

  return events;
}

/**
 * Generate markdown for a single module's events
 */
function generateModuleMarkdown(moduleName, events, descriptions, subsections) {
  const moduleDesc = descriptions?.module_description || `Events related to ${moduleName.toLowerCase()}.`;
  const eventDescriptions = descriptions?.events || {};

  let md = `## ${moduleName} Events\n\n`;
  md += `${moduleDesc}\n\n`;

  // Check if we have subsections defined
  if (subsections && Object.keys(subsections).length > 0) {
    for (const [subsectionName, eventNames] of Object.entries(subsections)) {
      const subsectionEvents = events.filter(e => eventNames.includes(e.name));
      if (subsectionEvents.length === 0) continue;

      md += `### ${subsectionName}\n\n`;
      md += generateEventsTable(subsectionEvents, eventDescriptions);
      md += '\n';
    }

    // Add any events not in subsections
    const subsectionedEvents = new Set(Object.values(subsections).flat());
    const remainingEvents = events.filter(e => !subsectionedEvents.has(e.name));
    if (remainingEvents.length > 0) {
      md += generateEventsTable(remainingEvents, eventDescriptions);
    }
  } else {
    md += generateEventsTable(events, eventDescriptions);
  }

  md += '\n---\n\n';
  return md;
}

/**
 * Generate a markdown table for events
 */
function generateEventsTable(events, eventDescriptions) {
  let md = '| Event | Description | Payload Fields |\n';
  md += '|-------|-------------|----------------|\n';

  for (const event of events) {
    const description = eventDescriptions[event.name] || 'No description available';
    const fields = event.fields.length > 0 ? '`' + event.fields.join('`, `') + '`' : '-';
    md += `| \`${event.name}\` | ${description} | ${fields} |\n`;
  }

  return md;
}

/**
 * Generate the full markdown document
 */
function generateMarkdown(schema, descriptions, locale) {
  const t = descriptions._translations || {};

  let md = `---
sidebar_position: 2
title: ${t.title || 'Domain Events'}
description: ${t.description || 'Public domain events published by Lana Bank'}
---

# ${t.title || 'Domain Events'}

${t.intro || 'Lana Bank publishes domain events via the transactional outbox pattern. These events can be consumed by external systems for integration, analytics, and audit purposes.'}

${t.serialization_note || 'All events are serialized as JSON and include metadata for tracing and ordering.'}

---

## ${t.event_structure || 'Event Structure'}

${t.event_structure_intro || 'Each event is wrapped in an envelope with the following structure:'}

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

  // Generate sections for each module
  for (const module of MODULE_ORDER) {
    const events = extractEventsFromSchema(schema, module.enumName);
    if (events.length === 0) {
      console.warn(`Warning: No events found for ${module.enumName}`);
      continue;
    }

    const moduleDescriptions = descriptions[module.enumName] || {};
    const subsections = moduleDescriptions.subsections || {};
    md += generateModuleMarkdown(module.key, events, moduleDescriptions, subsections);
  }

  // Event Types Reference
  md += `## ${t.event_types_reference || 'Event Types Reference'}

${t.event_types_intro || 'All event types follow the naming convention:'} \`core.<module>.<event-name>\`

| ${t.module || 'Module'} | ${t.event_type_prefix || 'Event Type Prefix'} |
|--------|-------------------|
`;

  for (const module of MODULE_ORDER) {
    md += `| ${module.key} | \`${module.prefix}.*\` |\n`;
  }

  md += `
---

## ${t.consuming_events || 'Consuming Events'}

${t.consuming_intro || 'Events are published via the transactional outbox and can be consumed through:'}

1. **${t.direct_polling || 'Direct database polling'}** - ${t.direct_polling_desc || 'Query the outbox table'}
2. **${t.event_streaming || 'Event streaming'}** - ${t.event_streaming_desc || 'Integration with message queues (implementation dependent)'}
3. **${t.etl_pipelines || 'ETL pipelines'}** - ${t.etl_pipelines_desc || 'Via Meltano extraction'}

${t.contact_note || 'For integration details, contact the platform team.'}
`;

  return md;
}

/**
 * Main function
 */
function main() {
  // Check if schema exists
  if (!fs.existsSync(SCHEMA_PATH)) {
    console.error(`Error: Schema file not found at ${SCHEMA_PATH}`);
    console.error('Run "make update-public-event-schemas" first to generate the schema.');
    process.exit(1);
  }

  // Load schema
  const schema = JSON.parse(fs.readFileSync(SCHEMA_PATH, 'utf-8'));

  // Load descriptions
  let descriptionsEn = {};
  let descriptionsEs = {};

  if (fs.existsSync(DESCRIPTIONS_EN_PATH)) {
    descriptionsEn = JSON.parse(fs.readFileSync(DESCRIPTIONS_EN_PATH, 'utf-8'));
  } else {
    console.warn(`Warning: English descriptions file not found at ${DESCRIPTIONS_EN_PATH}`);
  }

  if (fs.existsSync(DESCRIPTIONS_ES_PATH)) {
    descriptionsEs = JSON.parse(fs.readFileSync(DESCRIPTIONS_ES_PATH, 'utf-8'));
  } else {
    console.warn(`Warning: Spanish descriptions file not found at ${DESCRIPTIONS_ES_PATH}`);
  }

  // Generate English docs
  console.log('Generating English documentation...');
  const mdEn = generateMarkdown(schema, descriptionsEn, 'en');
  fs.writeFileSync(OUTPUT_EN_PATH, mdEn);
  console.log(`Written: ${OUTPUT_EN_PATH}`);

  // Generate Spanish docs
  console.log('Generating Spanish documentation...');
  const mdEs = generateMarkdown(schema, descriptionsEs, 'es');

  // Ensure directory exists
  const esDir = path.dirname(OUTPUT_ES_PATH);
  if (!fs.existsSync(esDir)) {
    fs.mkdirSync(esDir, { recursive: true });
  }
  fs.writeFileSync(OUTPUT_ES_PATH, mdEs);
  console.log(`Written: ${OUTPUT_ES_PATH}`);

  console.log('Done!');
}

main();
