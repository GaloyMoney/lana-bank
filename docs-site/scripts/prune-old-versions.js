#!/usr/bin/env node

/**
 * Prune old documentation versions, keeping at most MAX_VERSIONS.
 *
 * For each pruned version {ver}, removes:
 *   - versioned_docs/version-{ver}/
 *   - versioned_sidebars/version-{ver}-sidebars.json
 *   - schemas/versions/{ver}/
 *   - i18n/es/docusaurus-plugin-content-docs/version-{ver}/
 *   - i18n/es/docusaurus-plugin-content-docs/version-{ver}.json
 *   - The entry in versions.json
 */

const fs = require("fs");
const path = require("path");

const MAX_VERSIONS = 5;

const DOCS_SITE_DIR = path.join(__dirname, "..");
const VERSIONS_FILE = path.join(DOCS_SITE_DIR, "versions.json");

function rmSync(target) {
  if (!fs.existsSync(target)) return false;
  fs.rmSync(target, { recursive: true, force: true });
  return true;
}

function main() {
  if (!fs.existsSync(VERSIONS_FILE)) {
    console.log("No versions.json found, nothing to prune.");
    process.exit(0);
  }

  const versions = JSON.parse(fs.readFileSync(VERSIONS_FILE, "utf8"));

  if (versions.length <= MAX_VERSIONS) {
    console.log(
      `${versions.length} version(s) found, within limit of ${MAX_VERSIONS}. Nothing to prune.`
    );
    process.exit(0);
  }

  const toKeep = versions.slice(0, MAX_VERSIONS);
  const toRemove = versions.slice(MAX_VERSIONS);

  console.log(`Keeping versions: ${toKeep.join(", ")}`);
  console.log(`Pruning versions: ${toRemove.join(", ")}`);

  for (const ver of toRemove) {
    console.log(`\n--- Removing version ${ver} ---`);

    const targets = [
      path.join(DOCS_SITE_DIR, "versioned_docs", `version-${ver}`),
      path.join(
        DOCS_SITE_DIR,
        "versioned_sidebars",
        `version-${ver}-sidebars.json`
      ),
      path.join(DOCS_SITE_DIR, "schemas", "versions", ver),
      path.join(
        DOCS_SITE_DIR,
        "i18n",
        "es",
        "docusaurus-plugin-content-docs",
        `version-${ver}`
      ),
      path.join(
        DOCS_SITE_DIR,
        "i18n",
        "es",
        "docusaurus-plugin-content-docs",
        `version-${ver}.json`
      ),
    ];

    for (const target of targets) {
      const rel = path.relative(DOCS_SITE_DIR, target);
      if (rmSync(target)) {
        console.log(`  Removed: ${rel}`);
      } else {
        console.log(`  Not found (skipped): ${rel}`);
      }
    }
  }

  fs.writeFileSync(VERSIONS_FILE, JSON.stringify(toKeep, null, 2) + "\n");
  console.log(`\nUpdated versions.json: [${toKeep.join(", ")}]`);
  console.log("Done.");
}

main();
