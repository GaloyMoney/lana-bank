#!/usr/bin/env node

/**
 * Post-processing script that combines individual auto-generated API MDX files
 * into a single page per API (admin, customer).
 *
 * Run AFTER graphql-to-doc + fix-category-keys.js so descriptions are already injected.
 *
 * What it does:
 * 1. Reads all individual MDX files (operations + types)
 * 2. Extracts the meaningful content (strips duplicated React component boilerplate)
 * 3. Rewrites internal links to in-page anchors
 * 4. Generates one combined MDX file per API
 * 5. Removes the individual files/directories
 */

const fs = require("fs");
const path = require("path");

const DOCS_DIR = path.join(__dirname, "..", "docs");

// Categories to include and their source directories (relative to the API dir)
const CATEGORIES = [
  { name: "Queries", prefix: "query", dir: "operations/queries" },
  { name: "Mutations", prefix: "mutation", dir: "operations/mutations" },
  { name: "Subscriptions", prefix: "subscription", dir: "operations/subscriptions" },
  { name: "Directives (Operations)", prefix: "op-directive", dir: "operations/directives" },
  { name: "Objects", prefix: "object", dir: "types/objects" },
  { name: "Enums", prefix: "enum", dir: "types/enums" },
  { name: "Input Types", prefix: "input", dir: "types/inputs" },
  { name: "Scalars", prefix: "scalar", dir: "types/scalars" },
  { name: "Unions", prefix: "union", dir: "types/unions" },
  { name: "Directives (Types)", prefix: "type-directive", dir: "types/directives" },
];

// Shared React components used across all generated MDX files.
// Defined once at the top of the combined page.
const SHARED_COMPONENTS = `export const Bullet = () => <><span style={{ fontWeight: 'normal', fontSize: '.5em', color: 'var(--ifm-color-secondary-darkest)' }}>&nbsp;●&nbsp;</span></>

export const SpecifiedBy = (props) => <>Specification<a className="link" style={{ fontSize:'1.5em', paddingLeft:'4px' }} target="_blank" href={props.url} title={'Specified by ' + props.url}>⎘</a></>

export const Badge = (props) => <><span className={props.class}>{props.text}</span></>

import { useState } from 'react';

export const Details = ({ dataOpen, dataClose, children, startOpen = false }) => {
  const [open, setOpen] = useState(startOpen);
  return (
    <details {...(open ? { open: true } : {})} className="details" style={{ border:'none', boxShadow:'none', background:'var(--ifm-background-color)' }}>
      <summary
        onClick={(e) => {
          e.preventDefault();
          setOpen((open) => !open);
        }}
        style={{ listStyle:'none' }}
      >
      {open ? dataOpen : dataClose}
      </summary>
      {open && children}
    </details>
  );
};`;

/**
 * Extract the title from an MDX file's frontmatter and the actual content
 * (everything after the shared React component boilerplate).
 */
function extractContent(filePath) {
  const raw = fs.readFileSync(filePath, "utf8");
  const lines = raw.split("\n");

  // --- Extract title from frontmatter ---
  const titleMatch = raw.match(/^title:\s*(.+)$/m);
  const title = titleMatch ? titleMatch[1].trim() : path.basename(filePath, ".mdx");

  // --- Find end of frontmatter ---
  let fmCount = 0;
  let frontmatterEnd = 0;
  for (let i = 0; i < lines.length; i++) {
    if (lines[i].trim() === "---") {
      fmCount++;
      if (fmCount === 2) {
        frontmatterEnd = i + 1;
        break;
      }
    }
  }

  // --- Find end of React component boilerplate ---
  // The boilerplate ends with "};" (the closing of the Details export).
  // We search for the LAST "};" within the first ~40 lines after frontmatter.
  let boilerplateEnd = frontmatterEnd;
  const searchLimit = Math.min(frontmatterEnd + 45, lines.length);
  for (let i = frontmatterEnd; i < searchLimit; i++) {
    if (lines[i].trim() === "};") {
      boilerplateEnd = i + 1;
    }
  }

  // Skip blank lines after boilerplate
  while (boilerplateEnd < lines.length && lines[boilerplateEnd].trim() === "") {
    boilerplateEnd++;
  }

  const actualContent = lines.slice(boilerplateEnd).join("\n").trimEnd();
  return { title, content: actualContent };
}

/**
 * Read all MDX files in a directory and extract their content.
 * Returns an array of { title, content, path } sorted alphabetically by title.
 */
function readCategory(apiDir, relativeDir) {
  const catDir = path.join(apiDir, relativeDir);
  if (!fs.existsSync(catDir)) return [];

  return fs
    .readdirSync(catDir, { withFileTypes: true })
    .filter((e) => e.isFile() && e.name.endsWith(".mdx"))
    .map((e) => {
      const filePath = path.join(catDir, e.name);
      const extracted = extractContent(filePath);
      return { ...extracted, path: filePath };
    })
    .sort((a, b) => a.title.localeCompare(b.title));
}

/**
 * Build a mapping from every known file-path reference to its in-page anchor.
 *
 * Links in the generated MDX look like:
 *   (/apis/admin-api/types/objects/approval-process.mdx)
 *
 * We map these to anchors like:
 *   (#object-ApprovalProcess)
 */
function buildLinkMap(apiId, allCategories) {
  const map = {};
  const basePath = `/apis/${apiId}-api`;

  for (const cat of allCategories) {
    for (const file of cat.files) {
      // Derive the original URL path from the file's location on disk
      const relFromDocs = path.relative(DOCS_DIR, file.path).replace(/\\/g, "/");
      const urlPath = `/${relFromDocs}`;
      const anchor = `#${cat.prefix}-${file.title}`;

      map[urlPath] = anchor;
      // Also handle paths without .mdx extension (Docusaurus resolves both)
      map[urlPath.replace(/\.mdx$/, "")] = anchor;
    }
  }

  return map;
}

/**
 * Rewrite all internal links in `content` using the link map.
 */
function rewriteLinks(content, linkMap) {
  for (const [filePath, anchor] of Object.entries(linkMap)) {
    // Use split/join for literal replacement (no regex escaping needed)
    content = content.split(`(${filePath})`).join(`(${anchor})`);
  }
  return content;
}

/**
 * Remove a directory tree if it exists.
 */
function rmDir(dir) {
  if (fs.existsSync(dir)) {
    fs.rmSync(dir, { recursive: true });
    console.log(`  Removed ${path.relative(DOCS_DIR, dir)}/`);
  }
}

/**
 * Process one API: read individual files → generate combined page → cleanup.
 */
function processApi(apiId) {
  const apiDir = path.join(DOCS_DIR, "apis", `${apiId}-api`);
  if (!fs.existsSync(apiDir)) {
    console.log(`  Skipping ${apiId}: directory not found`);
    return;
  }

  const apiLabel = apiId.charAt(0).toUpperCase() + apiId.slice(1);
  console.log(`\nProcessing ${apiLabel} API...`);

  // 1. Read all files grouped by category
  const allCategories = CATEGORIES.map((cat) => ({
    ...cat,
    files: readCategory(apiDir, cat.dir),
  }));

  const totalFiles = allCategories.reduce((n, c) => n + c.files.length, 0);
  console.log(`  Found ${totalFiles} files across ${CATEGORIES.length} categories`);

  // 2. Build the link rewriting map
  const linkMap = buildLinkMap(apiId, allCategories);

  // 3. Assemble the combined page
  let page = `---
title: ${apiLabel} API Reference
slug: /apis/${apiId}-api
sidebar_label: ${apiLabel} API
pagination_next: null
pagination_prev: null
---

${SHARED_COMPONENTS}

This API reference has been automatically generated from the ${apiLabel} GraphQL schema.

`;

  for (const cat of allCategories) {
    if (cat.files.length === 0) continue;

    page += `## ${cat.name}\n\n`;

    for (let i = 0; i < cat.files.length; i++) {
      const file = cat.files[i];
      const anchorId = `${cat.prefix}-${file.title}`;

      page += `### \`${file.title}\` {#${anchorId}}\n\n`;
      page += rewriteLinks(file.content, linkMap) + "\n\n";

      if (i < cat.files.length - 1) {
        page += "---\n\n";
      }
    }

    page += "\n";
  }

  // 4. Write the combined page (replace the old generated.md landing page)
  const outputPath = path.join(apiDir, "api-reference.mdx");
  fs.writeFileSync(outputPath, page);
  console.log(`  Wrote ${path.relative(DOCS_DIR, outputPath)} (${(page.length / 1024).toFixed(0)} KB)`);

  // 5. Cleanup: remove individual file directories and the old landing page
  rmDir(path.join(apiDir, "operations"));
  rmDir(path.join(apiDir, "types"));

  const generatedPage = path.join(apiDir, "generated.md");
  if (fs.existsSync(generatedPage)) {
    fs.rmSync(generatedPage);
    console.log(`  Removed ${path.relative(DOCS_DIR, generatedPage)}`);
  }

  // Also remove stale _category_.yml at the API root
  const catYml = path.join(apiDir, "_category_.yml");
  if (fs.existsSync(catYml)) {
    fs.rmSync(catYml);
    console.log(`  Removed ${path.relative(DOCS_DIR, catYml)}`);
  }
}

function main() {
  console.log("Generating combined API reference pages...");
  processApi("admin");
  processApi("customer");
  console.log("\nDone!");
}

main();
