#!/usr/bin/env node

/**
 * Post-processing script for auto-generated API docs:
 * 1. Adds unique translation keys to _category_.yml files for i18n
 * 2. Adds Apollo Sandbox links to generated overview pages
 * 3. Injects "Try in Sandbox" buttons into each query/mutation page
 */

const fs = require("fs");
const path = require("path");
const yaml = require("js-yaml");

const DOCS_DIR = path.join(__dirname, "..", "docs");
const API_DIRS = ["api/admin", "api/customer"];

// GraphQL endpoint URLs for Apollo Sandbox links
// Update these when deploying to staging/production
const GRAPHQL_ENDPOINTS = {
  admin: "http://admin.localhost:4455/graphql",
  customer: "http://app.localhost:4455/graphql",
};

// Sandbox buttons are now injected into individual operation pages,
// so we no longer add them to overview pages.

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
}

main();
