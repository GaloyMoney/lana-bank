#!/usr/bin/env node

/**
 * Post-processing script to add unique translation keys to _category_.yml files
 * in auto-generated API docs. This prevents duplicate translation key errors
 * when building for multiple locales (e.g., Spanish).
 */

const fs = require("fs");
const path = require("path");
const yaml = require("js-yaml");

const DOCS_DIR = path.join(__dirname, "..", "docs");
const API_DIRS = ["api/admin", "api/customer"];

function updateGeneratedOverview(apiId, opts) {
  const filePath = path.join(DOCS_DIR, "api", apiId, "generated.md");
  if (!fs.existsSync(filePath)) return false;

  const content = fs.readFileSync(filePath, "utf8");

  // Very small frontmatter update: remove conflicting ids (no collisions) and
  // brand the page as Admin/Customer API (instead of generic "Schema Documentation").
  const updated = content
    .replace(/^id:\s*schema\s*\n/m, "")
    .replace(/^title:\s*Schema Documentation\s*$/m, `title: ${opts.title}`)
    .replace(
      /^This documentation has been automatically generated from the GraphQL schema\.\s*$/m,
      opts.lede
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
}

main();
