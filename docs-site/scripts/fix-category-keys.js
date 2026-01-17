#!/usr/bin/env node

/**
 * Post-processing script for auto-generated API docs:
 * 1. Adds unique translation keys to _category_.yml files for i18n
 * 2. Adds Apollo Sandbox links to generated overview pages
 * 3. Injects "Try in Sandbox" buttons into each query/mutation page
 * 4. Injects descriptions from api-descriptions.json into operation pages
 */

const fs = require("fs");
const path = require("path");
const yaml = require("js-yaml");

const DOCS_DIR = path.join(__dirname, "..", "docs");
const SCRIPTS_DIR = __dirname;
const I18N_ES_DOCS_DIR = path.join(__dirname, "..", "i18n", "es", "docusaurus-plugin-content-docs", "current");
const API_DIRS = ["api/admin", "api/customer"];
const DESCRIPTIONS_FILE = path.join(SCRIPTS_DIR, "api-descriptions.json");
const DESCRIPTIONS_ES_FILE = path.join(SCRIPTS_DIR, "api-descriptions.es.json");

// GraphQL endpoint URLs for Apollo Sandbox links
// Update these when deploying to staging/production
const GRAPHQL_ENDPOINTS = {
  admin: "http://admin.localhost:4455/graphql",
  customer: "http://app.localhost:4455/graphql",
};

// Sandbox buttons are now injected into individual operation pages,
// so we no longer add them to overview pages.

/**
 * Load the descriptions JSON file
 */
function loadDescriptions() {
  if (!fs.existsSync(DESCRIPTIONS_FILE)) {
    return {
      _meta: { version: "1.0", defaultDescription: "TODO: Add description" },
      admin: { queries: {}, mutations: {} },
      customer: { queries: {}, mutations: {} },
    };
  }
  return JSON.parse(fs.readFileSync(DESCRIPTIONS_FILE, "utf8"));
}

/**
 * Save the descriptions JSON file
 */
function saveDescriptions(descriptions) {
  fs.writeFileSync(DESCRIPTIONS_FILE, JSON.stringify(descriptions, null, 2) + "\n");
}

/**
 * Load the Spanish descriptions JSON file
 */
function loadSpanishDescriptions() {
  if (!fs.existsSync(DESCRIPTIONS_ES_FILE)) {
    return {
      _meta: { version: "1.0", defaultDescription: "TODO: Agregar descripcion" },
      admin: { queries: {}, mutations: {} },
      customer: { queries: {}, mutations: {} },
    };
  }
  return JSON.parse(fs.readFileSync(DESCRIPTIONS_ES_FILE, "utf8"));
}

/**
 * Save the Spanish descriptions JSON file
 */
function saveSpanishDescriptions(descriptions) {
  fs.writeFileSync(DESCRIPTIONS_ES_FILE, JSON.stringify(descriptions, null, 2) + "\n");
}

/**
 * Copy an operation file to Spanish i18n directory and inject Spanish description
 */
function copyToSpanishWithDescription(srcPath, apiId, operationType, descriptions) {
  const relativePath = path.relative(DOCS_DIR, srcPath);
  const destPath = path.join(I18N_ES_DOCS_DIR, relativePath);
  const destDir = path.dirname(destPath);

  // Create destination directory if it doesn't exist
  if (!fs.existsSync(destDir)) {
    fs.mkdirSync(destDir, { recursive: true });
  }

  // Read the English file (which already has sandbox buttons)
  let content = fs.readFileSync(srcPath, "utf8");

  // Get the operation name from filename
  const filename = path.basename(srcPath);
  const operationName = filenameToCamelCase(filename);

  // Get Spanish description
  const typeKey = operationType === "query" ? "queries" : "mutations";
  let description = descriptions[apiId]?.[typeKey]?.[operationName];
  let isNewOperation = false;

  if (!description) {
    description = descriptions._meta.defaultDescription;
    isNewOperation = true;
  }

  // Replace the English description with Spanish one
  // First find the English description pattern (after }; and before <TryInSandbox or ```graphql)
  const descPattern = /(\n\};)\n\n[^\n]+\n\n(<TryInSandbox|```graphql)/;
  if (descPattern.test(content)) {
    content = content.replace(descPattern, `$1\n\n${description}\n\n$2`);
  }

  fs.writeFileSync(destPath, content);
  return { updated: true, newOperation: isNewOperation ? operationName : null };
}

/**
 * Process Spanish operation files
 */
function processSpanishDescriptions(apiId, descriptions) {
  const apiDir = `api/${apiId}`;
  const files = findOperationFiles(apiDir);
  let updated = 0;
  const newOperations = { queries: [], mutations: [] };

  for (const file of files) {
    const typeKey = file.type === "query" ? "queries" : "mutations";
    const result = copyToSpanishWithDescription(file.path, apiId, file.type, descriptions);

    if (result.updated) {
      console.log(`  Created Spanish: ${path.relative(DOCS_DIR, file.path)}`);
      updated++;
    }

    if (result.newOperation) {
      newOperations[typeKey].push(result.newOperation);
    }
  }

  return { updated, newOperations };
}

/**
 * Convert kebab-case filename to camelCase operation name
 * e.g., "credit-facility-by-public-id" -> "creditFacilityByPublicId"
 */
function filenameToCamelCase(filename) {
  // Remove .mdx extension
  const name = filename.replace(/\.mdx$/, "");
  // Convert kebab-case to camelCase
  return name.replace(/-([a-z])/g, (_, letter) => letter.toUpperCase());
}

/**
 * Inject description into an operation MDX file
 * Returns: { updated: boolean, newOperation: string|null }
 */
function injectDescription(filePath, apiId, operationType, descriptions) {
  let content = fs.readFileSync(filePath, "utf8");
  const filename = path.basename(filePath);
  const operationName = filenameToCamelCase(filename);

  // Get description from JSON or use default
  const apiDescriptions = descriptions[apiId];
  const typeKey = operationType === "query" ? "queries" : "mutations";
  let description = apiDescriptions?.[typeKey]?.[operationName];
  let isNewOperation = false;

  if (!description) {
    // New operation - use default description and mark for addition
    description = descriptions._meta.defaultDescription;
    isNewOperation = true;
  }

  // Replace "No description" with actual description
  // The pattern handles multiple blank lines and possible TryInSandbox after
  // Pattern: }; followed by newlines, "No description", then newlines before <TryInSandbox or ```graphql
  const noDescPattern = /(\n\};)\n+No description\n+(<TryInSandbox|```graphql)/;
  const hasNoDesc = noDescPattern.test(content);

  if (hasNoDesc) {
    content = content.replace(noDescPattern, `$1\n\n${description}\n\n$2`);
    fs.writeFileSync(filePath, content);
    return { updated: true, newOperation: isNewOperation ? operationName : null };
  }

  // Also check for already injected descriptions that might need updating
  // This handles the case where default description needs to be replaced with real one
  const defaultDesc = descriptions._meta.defaultDescription;
  const defaultDescPattern = new RegExp(
    `(\\n\\};)\\n+${defaultDesc.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}\\n+(<TryInSandbox|\`\`\`graphql)`
  );

  if (defaultDescPattern.test(content) && !isNewOperation) {
    content = content.replace(defaultDescPattern, `$1\n\n${description}\n\n$2`);
    fs.writeFileSync(filePath, content);
    return { updated: true, newOperation: null };
  }

  return { updated: false, newOperation: isNewOperation ? operationName : null };
}

/**
 * Process descriptions for all operation files in an API
 */
function processDescriptions(apiId, descriptions) {
  const apiDir = `api/${apiId}`;
  const files = findOperationFiles(apiDir);
  let updated = 0;
  const newOperations = { queries: [], mutations: [] };

  for (const file of files) {
    const typeKey = file.type === "query" ? "queries" : "mutations";
    const result = injectDescription(file.path, apiId, file.type, descriptions);

    if (result.updated) {
      console.log(`  Updated description: ${path.relative(DOCS_DIR, file.path)}`);
      updated++;
    }

    if (result.newOperation) {
      newOperations[typeKey].push(result.newOperation);
    }
  }

  return { updated, newOperations };
}

function updateGeneratedOverview(apiId, opts) {
  const filePath = path.join(DOCS_DIR, "api", apiId, "generated.md");
  if (!fs.existsSync(filePath)) return false;

  let content = fs.readFileSync(filePath, "utf8");

  // Very small frontmatter update: remove conflicting ids (no collisions) and
  // brand the page as Admin/Customer API (instead of generic "Schema Documentation").
  let updated = content
    .replace(/^id:\s*schema\s*\n/m, "")
    .replace(/^title:\s*Schema Documentation\s*$/m, `title: ${opts.title}`)
    .replace(
      /^This documentation has been automatically generated from the GraphQL schema\.\s*$/m,
      opts.lede
    );

  // Remove any existing Interactive Explorer section (sandbox buttons are now on individual pages)
  updated = updated.replace(
    /\n## Interactive Explorer[\s\S]*?(?=\n## |$)/,
    ""
  );

  if (updated === content) return false;
  fs.writeFileSync(filePath, updated);
  console.log(`  Updated ${path.relative(DOCS_DIR, filePath)} (title/id/lede)`);
  return true;
}

function findCategoryFiles(dir, files = []) {
  const entries = fs.readdirSync(dir, { withFileTypes: true });

  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      findCategoryFiles(fullPath, files);
    } else if (entry.name === "_category_.yml" || entry.name === "_category_.json") {
      files.push(fullPath);
    }
  }

  return files;
}

function generateUniqueKey(filePath) {
  // Extract path relative to docs dir and create a unique key
  const relativePath = path.relative(DOCS_DIR, path.dirname(filePath));
  // Convert path separators to dashes and make it a valid key
  return relativePath.replace(/[\/\\]/g, "-").toLowerCase();
}

function processYmlFile(filePath) {
  const content = fs.readFileSync(filePath, "utf8");
  const data = yaml.load(content);

  // Skip if key already exists
  if (data.key) {
    console.log(`  Skipping ${filePath} (key already exists)`);
    return false;
  }

  data.key = generateUniqueKey(filePath);

  const newContent = yaml.dump(data, {
    lineWidth: -1,
    quotingType: '"',
    forceQuotes: false
  });
  fs.writeFileSync(filePath, newContent);
  console.log(`  Updated ${filePath} with key: ${data.key}`);
  return true;
}

function processJsonFile(filePath) {
  const content = fs.readFileSync(filePath, "utf8");
  const data = JSON.parse(content);

  // Skip if key already exists
  if (data.key) {
    console.log(`  Skipping ${filePath} (key already exists)`);
    return false;
  }

  data.key = generateUniqueKey(filePath);
  fs.writeFileSync(filePath, JSON.stringify(data, null, 2) + "\n");
  console.log(`  Updated ${filePath} with key: ${data.key}`);
  return true;
}

/**
 * Find all operation files (queries and mutations) in an API directory
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
          path: path.join(dir, entry.name),
          type: subdir === "queries" ? "query" : "mutation",
        });
      }
    }
  }

  return files;
}

/**
 * Extract the GraphQL operation from a generated .mdx file
 */
function extractGraphQLOperation(content, operationType) {
  // The operation is in a ```graphql code block
  const graphqlMatch = content.match(/```graphql\n([\s\S]*?)```/);
  if (!graphqlMatch) return null;

  const operationSignature = graphqlMatch[1].trim();

  // Build a full operation with the signature
  // The signature looks like: operationName(args): ReturnType!
  // We need to wrap it in query/mutation { ... }
  const operationName = operationSignature.split("(")[0].trim();

  // Build a basic operation template
  // For mutations: mutation { operationName(input: $input) { ... } }
  // For queries: query { operationName(...) { ... } }
  return {
    name: operationName,
    signature: operationSignature,
  };
}

/**
 * Generate the full GraphQL operation string for Apollo Sandbox
 */
function buildGraphQLOperation(operationType, operationName, signature) {
  // Parse the signature to extract arguments
  const argsMatch = signature.match(/\(([\s\S]*?)\)/);
  const returnTypeMatch = signature.match(/\):\s*([\s\S]+)$/);

  let argsStr = "";
  let variableDefinitions = "";

  if (argsMatch && argsMatch[1].trim()) {
    // Parse arguments like: input: SomeInput!, id: ID!
    const args = argsMatch[1].split(",").map((a) => a.trim());
    const argParts = [];
    const varParts = [];

    for (const arg of args) {
      const [argName, argType] = arg.split(":").map((s) => s.trim());
      if (argName && argType) {
        argParts.push(`${argName}: $${argName}`);
        varParts.push(`$${argName}: ${argType}`);
      }
    }

    if (argParts.length > 0) {
      argsStr = `(${argParts.join(", ")})`;
      variableDefinitions = `(${varParts.join(", ")})`;
    }
  }

  // Build the operation
  const opType = operationType === "mutation" ? "mutation" : "query";
  const opName = operationName.charAt(0).toUpperCase() + operationName.slice(1);

  return `${opType} ${opName}${variableDefinitions} {
  ${operationName}${argsStr} {
    # Add fields here
    __typename
  }
}`;
}

/**
 * Inject "Try in Sandbox" button into an operation .mdx file
 */
function injectSandboxButton(filePath, apiId) {
  let content = fs.readFileSync(filePath, "utf8");

  // Skip if already has sandbox button
  if (content.includes("TryInSandbox")) {
    return false;
  }

  const endpoint = GRAPHQL_ENDPOINTS[apiId];
  if (!endpoint) return false;

  // Determine operation type from path
  const operationType = filePath.includes("/mutations/") ? "mutation" : "query";

  // Extract the operation
  const operation = extractGraphQLOperation(content, operationType);
  if (!operation) return false;

  // Build the full GraphQL operation for the sandbox
  const fullOperation = buildGraphQLOperation(
    operationType,
    operation.name,
    operation.signature
  );

  // Create the import and component injection
  const importStatement = `import TryInSandbox from '@site/src/components/TryInSandbox';\n`;

  // Find the position after the frontmatter and inline component definitions
  // We want to insert after the "No description" or description text, before the graphql block
  const graphqlBlockIndex = content.indexOf("```graphql");
  if (graphqlBlockIndex === -1) return false;

  // Find a good insertion point - after description, before the code block
  // Look for the last export statement or "No description" before the graphql block
  const beforeGraphql = content.substring(0, graphqlBlockIndex);
  let insertIndex = graphqlBlockIndex;

  // Find where the component definitions end (after the last export const Details)
  const detailsEndMatch = beforeGraphql.match(/export const Details[\s\S]*?\n\};/);
  if (detailsEndMatch) {
    insertIndex = beforeGraphql.lastIndexOf(detailsEndMatch[0]) + detailsEndMatch[0].length;
  }

  // Build the sandbox button JSX
  // We need to escape the operation string for JSX
  const escapedOperation = fullOperation
    .replace(/\\/g, "\\\\")
    .replace(/`/g, "\\`")
    .replace(/\$/g, "\\$");

  const sandboxComponent = `

<TryInSandbox
  endpoint="${endpoint}"
  operationName="${operation.name}"
  operation={\`${escapedOperation}\`}
/>

`;

  // Insert the import at the top (after frontmatter) and component before graphql block
  // First, add the import after the frontmatter
  const frontmatterEnd = content.indexOf("---", 4) + 3;
  const afterFrontmatter = content.substring(frontmatterEnd);

  // Check if import already exists
  if (!content.includes("import TryInSandbox")) {
    content = content.substring(0, frontmatterEnd) + "\n" + importStatement + afterFrontmatter;
  }

  // Now find the graphql block again (index may have shifted)
  const newGraphqlIndex = content.indexOf("```graphql");

  // Insert the sandbox button right before the graphql block
  content =
    content.substring(0, newGraphqlIndex) +
    sandboxComponent +
    content.substring(newGraphqlIndex);

  fs.writeFileSync(filePath, content);
  return true;
}

/**
 * Process all operation files in an API directory
 */
function processOperationFiles(apiId) {
  const apiDir = `api/${apiId}`;
  const files = findOperationFiles(apiDir);
  let updated = 0;

  for (const file of files) {
    if (injectSandboxButton(file.path, apiId)) {
      console.log(`  Added sandbox button: ${path.relative(DOCS_DIR, file.path)}`);
      updated++;
    }
  }

  return updated;
}

function main() {
  console.log("Adding unique keys to _category_ files...\n");

  let totalUpdated = 0;

  for (const apiDir of API_DIRS) {
    const fullDir = path.join(DOCS_DIR, apiDir);

    if (!fs.existsSync(fullDir)) {
      console.log(`Directory not found: ${fullDir}`);
      continue;
    }

    console.log(`Processing ${apiDir}...`);
    const categoryFiles = findCategoryFiles(fullDir);

    for (const filePath of categoryFiles) {
      const ext = path.extname(filePath);
      let updated = false;

      if (ext === ".yml") {
        updated = processYmlFile(filePath);
      } else if (ext === ".json") {
        updated = processJsonFile(filePath);
      }

      if (updated) totalUpdated++;
    }
  }

  console.log(`\nDone! Updated ${totalUpdated} category files.`);

  console.log("\nNormalizing generated overview pages...\n");
  updateGeneratedOverview("admin", {
    title: "Admin API",
    lede: "This documentation has been automatically generated from the Admin GraphQL schema.",
  });
  updateGeneratedOverview("customer", {
    title: "Customer API",
    lede:
      "This documentation has been automatically generated from the Customer GraphQL schema.",
  });

  console.log("\nAdding 'Try in Sandbox' buttons to operation pages...\n");
  let operationsUpdated = 0;
  operationsUpdated += processOperationFiles("admin");
  operationsUpdated += processOperationFiles("customer");
  console.log(`\nDone! Added sandbox buttons to ${operationsUpdated} operation pages.`);

  // Process descriptions
  console.log("\nInjecting API descriptions...\n");
  const descriptions = loadDescriptions();
  let descriptionsUpdated = 0;
  const allNewOperations = { admin: { queries: [], mutations: [] }, customer: { queries: [], mutations: [] } };

  for (const apiId of ["admin", "customer"]) {
    console.log(`Processing ${apiId} API descriptions...`);
    const result = processDescriptions(apiId, descriptions);
    descriptionsUpdated += result.updated;
    allNewOperations[apiId] = result.newOperations;
  }

  // Add new operations to descriptions JSON with default description
  let newOpsAdded = 0;
  for (const apiId of ["admin", "customer"]) {
    for (const typeKey of ["queries", "mutations"]) {
      for (const opName of allNewOperations[apiId][typeKey]) {
        if (!descriptions[apiId][typeKey][opName]) {
          descriptions[apiId][typeKey][opName] = descriptions._meta.defaultDescription;
          console.log(`  Added placeholder for new operation: ${apiId}.${typeKey}.${opName}`);
          newOpsAdded++;
        }
      }
    }
  }

  if (newOpsAdded > 0) {
    saveDescriptions(descriptions);
    console.log(`\nAdded ${newOpsAdded} new operations to api-descriptions.json with placeholder descriptions.`);
  }

  console.log(`\nDone! Updated ${descriptionsUpdated} operation descriptions.`);

  // Process Spanish descriptions
  console.log("\nCreating Spanish API operation pages...\n");
  const spanishDescriptions = loadSpanishDescriptions();
  let spanishUpdated = 0;
  const allNewSpanishOperations = { admin: { queries: [], mutations: [] }, customer: { queries: [], mutations: [] } };

  for (const apiId of ["admin", "customer"]) {
    console.log(`Processing ${apiId} API Spanish descriptions...`);
    const result = processSpanishDescriptions(apiId, spanishDescriptions);
    spanishUpdated += result.updated;
    allNewSpanishOperations[apiId] = result.newOperations;
  }

  // Add new operations to Spanish descriptions JSON with default description
  let newSpanishOpsAdded = 0;
  for (const apiId of ["admin", "customer"]) {
    for (const typeKey of ["queries", "mutations"]) {
      for (const opName of allNewSpanishOperations[apiId][typeKey]) {
        if (!spanishDescriptions[apiId][typeKey][opName]) {
          spanishDescriptions[apiId][typeKey][opName] = spanishDescriptions._meta.defaultDescription;
          console.log(`  Added Spanish placeholder for new operation: ${apiId}.${typeKey}.${opName}`);
          newSpanishOpsAdded++;
        }
      }
    }
  }

  if (newSpanishOpsAdded > 0) {
    saveSpanishDescriptions(spanishDescriptions);
    console.log(`\nAdded ${newSpanishOpsAdded} new operations to api-descriptions.es.json with placeholder descriptions.`);
  }

  console.log(`\nDone! Created ${spanishUpdated} Spanish operation pages.`);
}

main();
