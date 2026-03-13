import { readdirSync, readFileSync, existsSync } from "fs"
import { join, relative, dirname } from "path"

const ROOT = join(import.meta.dirname, "..")

// ── Load and merge translations ─────────────────────────────────────────────

function loadJson(path) {
  return JSON.parse(readFileSync(path, "utf8"))
}

const base = loadJson(join(ROOT, "messages", "en.json"))
const generated = loadJson(join(ROOT, "messages", "generated", "en.json"))
const merged = { ...base, ...generated }

/** Flatten nested object to dot-path keys. Includes both leaves and intermediate paths. */
function collectPaths(obj, prefix = "") {
  const paths = new Set()
  for (const key of Object.keys(obj)) {
    const full = prefix ? `${prefix}.${key}` : key
    paths.add(full)
    if (typeof obj[key] === "object" && obj[key] !== null) {
      for (const p of collectPaths(obj[key], full)) paths.add(p)
    }
  }
  return paths
}

const validKeys = collectPaths(merged)

/** Get direct child keys of a dot-path in the merged translations. */
function getChildKeys(dotPath) {
  const parts = dotPath.split(".")
  let obj = merged
  for (const p of parts) {
    if (obj == null || typeof obj !== "object") return null
    obj = obj[p]
  }
  if (obj == null || typeof obj !== "object") return null
  return Object.keys(obj)
}

/** Resolve a dot-path to the translation value. */
function resolvePath(dotPath) {
  const parts = dotPath.split(".")
  let obj = merged
  for (const p of parts) {
    if (obj == null || typeof obj !== "object") return undefined
    obj = obj[p]
  }
  return obj
}

// ── Parse GraphQL enums ─────────────────────────────────────────────────────

function parseGraphQLEnums() {
  const enumFile = join(ROOT, "lib", "graphql", "generated", "index.ts")
  const src = readFileSync(enumFile, "utf8")
  const enumMap = new Map()

  const enumRe = /export\s+enum\s+(\w+)\s*\{([^}]+)\}/g
  for (const m of src.matchAll(enumRe)) {
    const name = m[1]
    const body = m[2]
    const memberNames = new Set()
    const values = new Set()
    const memberRe = /(\w+)\s*=\s*'([^']+)'/g
    for (const mv of body.matchAll(memberRe)) {
      memberNames.add(mv[1])
      values.add(mv[2])
    }
    if (values.size > 0) enumMap.set(name, { memberNames, values })
  }
  return enumMap
}

const graphqlEnums = parseGraphQLEnums()

function lowerFirst(s) {
  return s.charAt(0).toLowerCase() + s.slice(1)
}

const enumCandidates = new Map()
for (const [name, { memberNames, values }] of graphqlEnums) {
  const candidates = []
  const seen = new Set()
  const addCandidate = (strategy, keys) => {
    const sig = [...keys].sort().join(",")
    if (!seen.has(sig)) {
      seen.add(sig)
      candidates.push({ strategy, keys })
    }
  }
  addCandidate("lowercased values", new Set([...values].map((v) => v.toLowerCase())))
  addCandidate("camelCase members", new Set([...memberNames].map(lowerFirst)))
  addCandidate("raw values", new Set(values))
  enumCandidates.set(name, candidates)
}

// ── Scan source files ───────────────────────────────────────────────────────

const EXCLUDED_DIRS = new Set(["node_modules", ".next", "generated", "i18n"])

function walk(dir) {
  const files = []
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    if (EXCLUDED_DIRS.has(entry.name)) continue
    const full = join(dir, entry.name)
    if (entry.isDirectory()) {
      files.push(...walk(full))
    } else if (/\.tsx?$/.test(entry.name)) {
      files.push(full)
    }
  }
  return files
}

const sourceFiles = walk(ROOT)

// ── Helpers ─────────────────────────────────────────────────────────────────

const errors = []
const warnings = []

function escapeRegExp(s) {
  return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")
}

/**
 * Extract all string literal return values from a function body.
 * Handles: return "literal", return 'literal'
 */
function extractReturnLiterals(fnBody) {
  const literals = new Set()
  const re = /return\s+["']([^"']+)["']/g
  for (const m of fnBody.matchAll(re)) {
    literals.add(m[1])
  }
  return literals
}

/**
 * Find a function definition in source text and return its body.
 * Handles: function name(...) { ... } and const name = (...) => { ... }
 */
function findFunctionBody(src, fnName) {
  // Try: function fnName(...)  { ... }
  const fnRe = new RegExp(
    `(?:function\\s+${escapeRegExp(fnName)}|(?:const|let|var)\\s+${escapeRegExp(fnName)}\\s*=\\s*(?:(?:\\([^)]*\\)|\\w+)\\s*(?::\\s*[^=]+)?\\s*=>))`,
  )
  const match = fnRe.exec(src)
  if (!match) return null

  // Find the opening brace
  let pos = match.index + match[0].length
  while (pos < src.length && src[pos] !== "{") pos++
  if (pos >= src.length) return null

  // Match braces to find the end
  let depth = 0
  const start = pos
  for (; pos < src.length; pos++) {
    if (src[pos] === "{") depth++
    else if (src[pos] === "}") {
      depth--
      if (depth === 0) return src.slice(start, pos + 1)
    }
  }
  return null
}

/**
 * Try to verify a dynamic key against GraphQL enums.
 * Returns true if handled.
 */
function tryVerifyDynamicKeyEnum(rel, namespaces, prefix) {
  for (const ns of namespaces) {
    const fullPrefix = `${ns}.${prefix}`
    const childKeys = getChildKeys(fullPrefix)
    if (!childKeys || childKeys.length === 0) continue
    const childSet = new Set(childKeys)

    for (const [enumName, candidates] of enumCandidates) {
      for (const { strategy, keys: enumKeys } of candidates) {
        const matching = [...enumKeys].filter((v) => childSet.has(v))
        if (matching.length < enumKeys.size * 0.8) continue
        const missing = [...enumKeys].filter((v) => !childSet.has(v))
        if (missing.length > 0) {
          for (const m of missing) {
            errors.push(
              `${rel}: missing key "${fullPrefix}.${m}" (enum ${enumName} via ${strategy})`,
            )
          }
        }
        return true
      }
    }
  }
  return false
}

/**
 * Try to verify a dynamic key by extracting function return values.
 * For t(`prefix.${fnCall(...)}`), find fnName in src and extract return literals.
 */
function tryVerifyFunctionReturns(rel, src, file, namespaces, prefix, expr) {
  // Extract function name from expr like "getStatusKey(status)" or "validateTermsFields({...})"
  const fnCallMatch = expr.match(/^(\w+)\s*\(/)
  if (!fnCallMatch) return false
  const fnName = fnCallMatch[1]

  // First try to find the function in the same file
  let fnBody = findFunctionBody(src, fnName)

  // If not found locally, try to resolve imports
  if (!fnBody) {
    const importRe = new RegExp(
      `import\\s*\\{[^}]*\\b${escapeRegExp(fnName)}\\b[^}]*\\}\\s*from\\s*["']([^"']+)["']`,
    )
    const importMatch = importRe.exec(src)
    if (importMatch) {
      const importPath = importMatch[1]
      // Resolve relative to file, try .ts and .tsx extensions
      const dir = dirname(file)
      for (const ext of ["", ".ts", ".tsx", "/index.ts", "/index.tsx"]) {
        const resolved = join(dir, importPath + ext)
        if (existsSync(resolved)) {
          const importedSrc = readFileSync(resolved, "utf8")
          fnBody = findFunctionBody(importedSrc, fnName)
          if (fnBody) break
        }
      }
      // Also try resolving from ROOT for @/ imports
      if (!fnBody && importPath.startsWith("@/")) {
        const aliasPath = importPath.replace("@/", "")
        for (const ext of ["", ".ts", ".tsx", "/index.ts", "/index.tsx"]) {
          const resolved = join(ROOT, aliasPath + ext)
          if (existsSync(resolved)) {
            const importedSrc = readFileSync(resolved, "utf8")
            fnBody = findFunctionBody(importedSrc, fnName)
            if (fnBody) break
          }
        }
      }
    }
  }

  if (!fnBody) return false

  const returnValues = extractReturnLiterals(fnBody)
  if (returnValues.size === 0) return false

  for (const val of returnValues) {
    const found = [...namespaces].some((ns) => validKeys.has(`${ns}.${prefix}.${val}`))
    if (!found) {
      const tried = [...namespaces].map((ns) => `${ns}.${prefix}.${val}`).join(", ")
      errors.push(`${rel}: missing key (tried: ${tried}) [from ${fnName}() return value]`)
    }
  }
  return true
}

/**
 * Parse GraphQL query field names from gql template literals in source.
 * Handles nested queries by extracting field blocks at all depths.
 * Returns Map<blockName, Set<leafFieldName>>.
 */
function parseGqlQueryFields(src) {
  const queries = new Map()
  const gqlRe = /gql\s*`([^`]+)`/g
  for (const m of src.matchAll(gqlRe)) {
    parseGqlBlocks(m[1], queries)
  }
  return queries
}

/** Recursively parse { ... } blocks in a GraphQL document. */
function parseGqlBlocks(text, result) {
  // Find all "name { ... }" blocks, handling nesting via brace matching
  const re = /(\w+)\s*(?:\([^)]*\)\s*)?\{/g
  let match
  while ((match = re.exec(text)) !== null) {
    const blockName = match[1]
    const braceStart = match.index + match[0].length - 1
    const body = extractBraceContent(text, braceStart)
    if (body === null) continue

    // Skip GraphQL operation keywords
    if (["query", "mutation", "fragment", "subscription", "on"].includes(blockName)) {
      // Still recurse into the body to find nested blocks
      parseGqlBlocks(body, result)
      continue
    }

    // Extract leaf fields (no sub-blocks) from this level
    const fields = new Set()
    // Remove nested blocks from body to get only leaf fields
    const withoutBlocks = body.replace(/\w+\s*(?:\([^)]*\)\s*)?\{[^]*?\}/g, "")
    for (const line of withoutBlocks.split(/[\n,]/)) {
      const trimmed = line.trim()
      if (/^\w+$/.test(trimmed) && trimmed !== "__typename") {
        fields.add(trimmed)
      }
    }
    if (fields.size > 0) result.set(blockName, fields)

    // Recurse into nested blocks
    parseGqlBlocks(body, result)
  }
}

/** Extract content between matching braces starting at position of '{'. */
function extractBraceContent(text, startPos) {
  if (text[startPos] !== "{") return null
  let depth = 0
  for (let i = startPos; i < text.length; i++) {
    if (text[i] === "{") depth++
    else if (text[i] === "}") {
      depth--
      if (depth === 0) return text.slice(startPos + 1, i)
    }
  }
  return null
}

/**
 * For a variable used in a template literal, try to trace it back to a
 * function call assignment. e.g.:
 *   const validationError = validateTermsFields(...)
 *   t(`errors.${validationError}`)
 * Returns the function name if found, null otherwise.
 */
function traceVariableToFunctionCall(src, varName) {
  // Look for: const/let varName = functionName(...)
  const re = new RegExp(
    `(?:const|let|var)\\s+${escapeRegExp(varName)}\\s*=\\s*(\\w+)\\s*\\(`,
  )
  const m = re.exec(src)
  return m ? m[1] : null
}

// ── Pass 1: Per-file static and dynamic key verification ────────────────────

// Also collect data for later passes
const dynamicNamespaceFiles = [] // Files with useTranslations(variable)
const dynamicKeyWarnings = [] // Track warnings to potentially resolve later

for (const file of sourceFiles) {
  const src = readFileSync(file, "utf8")
  const rel = relative(ROOT, file)

  const nsMap = new Map()

  const assignRe = /\b(?:const|let|var)\s+(\w+)\s*=\s*useTranslations\(\s*"([^"]+)"\s*\)/g
  for (const m of src.matchAll(assignRe)) {
    if (!nsMap.has(m[1])) nsMap.set(m[1], new Set())
    nsMap.get(m[1]).add(m[2])
  }

  // Track dynamic namespace usage
  const dynNsRe = /useTranslations\(\s*([a-zA-Z_]\w*)\s*\)/g
  for (const m of src.matchAll(dynNsRe)) {
    dynamicNamespaceFiles.push({ file, rel, varName: m[1] })
  }

  // Immediate invocation
  const immediateRe = /useTranslations\(\s*"([^"]+)"\s*\)\s*\(\s*"([^"]+)"\s*\)/g
  for (const m of src.matchAll(immediateRe)) {
    const fullKey = `${m[1]}.${m[2]}`
    if (!validKeys.has(fullKey)) {
      errors.push(`${rel}: missing key "${fullKey}" (immediate invocation)`)
    }
  }

  for (const [varName, namespaces] of nsMap) {
    // Static key calls
    const callRe = new RegExp(
      `\\b${escapeRegExp(varName)}(?:\\.rich)?\\(\\s*"([^"]+)"`,
      "g",
    )
    for (const m of src.matchAll(callRe)) {
      const key = m[1]
      const found = [...namespaces].some((ns) => validKeys.has(`${ns}.${key}`))
      if (!found) {
        const tried = [...namespaces].map((ns) => `${ns}.${key}`).join(", ")
        errors.push(`${rel}: missing key (tried: ${tried})`)
      }
    }

    // Dynamic key calls
    const tplRe = new RegExp(
      `\\b${escapeRegExp(varName)}(?:\\.rich)?\\(\\s*\`` +
        `([^$\`]*?)` +
        `\\$\\{([^}]*?)\\}` +
        `([^$\`]*?)` + // capture suffix after }
        `\``,
      "g",
    )
    for (const m of src.matchAll(tplRe)) {
      const rawPrefix = m[1]
      const expr = m[2].trim()
      const suffix = m[3].trim()
      const prefix = rawPrefix.endsWith(".") ? rawPrefix.slice(0, -1) : rawPrefix

      const isSimpleExpr = /^[\w.?]+(?:\.(?:toLowerCase|toLocaleLowerCase)\(\))?$/.test(expr)
      const isFunctionCall = /^\w+\s*\(/.test(expr)
      const isSimpleVariable = /^\w+$/.test(expr)

      if (prefix && !suffix) {
        let handled = false

        // Strategy 1: Enum matching (for simple expressions)
        if (isSimpleExpr) {
          handled = tryVerifyDynamicKeyEnum(rel, namespaces, prefix)
        }

        // Strategy 2: Function return value extraction (direct call)
        if (!handled && isFunctionCall) {
          handled = tryVerifyFunctionReturns(rel, src, file, namespaces, prefix, expr)
        }

        // Strategy 2b: Variable traced back to a function call
        if (!handled && isSimpleVariable) {
          const tracedFn = traceVariableToFunctionCall(src, expr)
          if (tracedFn) {
            handled = tryVerifyFunctionReturns(
              rel, src, file, namespaces, prefix, `${tracedFn}()`,
            )
          }
        }

        // Strategy 3: GraphQL field name matching (for Object.entries patterns)
        // Only applies when the expression is a simple variable (like `key` from
        // Object.entries destructuring), not property access or method calls.
        if (!handled && isSimpleVariable) {
          const gqlFields = parseGqlQueryFields(src)
          // Look for Object.entries pattern referencing this prefix
          // The prefix in the translation corresponds to a GraphQL query field
          for (const [queryField, fields] of gqlFields) {
            // Check if the prefix matches the query field name
            // e.g., prefix="deposit" matches query field "depositConfig" → inner fields
            if (
              prefix === queryField ||
              prefix + "Config" === queryField ||
              queryField === prefix
            ) {
              for (const field of fields) {
                const found = [...namespaces].some((ns) =>
                  validKeys.has(`${ns}.${prefix}.${field}`),
                )
                if (!found) {
                  const tried = [...namespaces]
                    .map((ns) => `${ns}.${prefix}.${field}`)
                    .join(", ")
                  errors.push(`${rel}: missing key (tried: ${tried}) [from GraphQL query field]`)
                }
              }
              handled = true
              break
            }
          }
        }

        if (!handled) {
          const prefixExists = [...namespaces].some((ns) => validKeys.has(`${ns}.${prefix}`))
          if (!prefixExists) {
            const tried = [...namespaces].map((ns) => `${ns}.${prefix}`).join(", ")
            errors.push(`${rel}: missing translation prefix (tried: ${tried})`)
          } else {
            warnings.push(
              `${rel}: dynamic key ${varName}(\`${rawPrefix}...\`) — cannot match to enum`,
            )
          }
        }
      } else if (prefix && suffix) {
        // Pattern like t(`${expr}.suffix`) or t(`prefix.${expr}.suffix`)
        // Check if suffix is a known sub-key pattern
        const cleanSuffix = suffix.startsWith(".") ? suffix.slice(1) : suffix
        if (cleanSuffix === "label" || cleanSuffix === "description") {
          // Permission-style pattern: t(`${name}.label`) / t(`${name}.description`)
          // Will be handled in Pass 4
        } else {
          warnings.push(
            `${rel}: dynamic key ${varName}(\`${rawPrefix}\${...}${suffix}\`) — cannot verify`,
          )
        }
      } else if (!prefix && suffix) {
        // Pattern like t(`${expr}.suffix`)
        const cleanSuffix = suffix.startsWith(".") ? suffix.slice(1) : suffix
        if (cleanSuffix === "label" || cleanSuffix === "description") {
          // Handled in Pass 4 (permission completeness)
        } else {
          warnings.push(
            `${rel}: dynamic key ${varName}(\`\${...}${suffix}\`) — cannot verify statically`,
          )
        }
      } else {
        warnings.push(`${rel}: dynamic key ${varName}(\`\${...}\`) — cannot verify statically`)
      }
    }
  }
}

// ── Pass 2: Dynamic namespace resolution ────────────────────────────────────
// For useTranslations(variable), find all callers that pass literal strings

for (const { file, rel, varName } of dynamicNamespaceFiles) {
  const src = readFileSync(file, "utf8")

  // Find the component name that accepts this variable as a prop
  // Pattern: type Props = { translationNamespace: string } or similar
  // Then find all JSX usages: <Component translationNamespace="Literal" />
  const propRe = new RegExp(`${escapeRegExp(varName)}\\s*:\\s*string`)
  if (!propRe.test(src)) continue

  // Find keys used with this variable via t("key") calls
  // Find the t variable assigned from useTranslations(varName)
  const tAssignRe = new RegExp(
    `\\b(?:const|let|var)\\s+(\\w+)\\s*=\\s*useTranslations\\(\\s*${escapeRegExp(varName)}\\s*\\)`,
  )
  const tMatch = tAssignRe.exec(src)
  if (!tMatch) continue
  const tVar = tMatch[1]

  // Extract all static keys used
  const keyRe = new RegExp(`\\b${escapeRegExp(tVar)}\\(\\s*"([^"]+)"`, "g")
  const usedKeys = new Set()
  for (const km of src.matchAll(keyRe)) {
    usedKeys.add(km[1])
  }
  if (usedKeys.size === 0) continue

  // Find all callers across the codebase that pass this prop as a literal
  const callerNamespaces = new Set()
  for (const sf of sourceFiles) {
    const callerSrc = readFileSync(sf, "utf8")
    // Match: translationNamespace="LiteralString" or varName="LiteralString"
    const callerRe = new RegExp(`${escapeRegExp(varName)}\\s*=\\s*"([^"]+)"`, "g")
    for (const cm of callerSrc.matchAll(callerRe)) {
      callerNamespaces.add(cm[1])
    }
  }

  // Verify each caller namespace + used key
  for (const ns of callerNamespaces) {
    for (const key of usedKeys) {
      if (!validKeys.has(`${ns}.${key}`)) {
        const callerRel = relative(ROOT, file)
        errors.push(`${callerRel}: missing key "${ns}.${key}" (dynamic namespace from callers)`)
      }
    }
  }
}

// ── Pass 3: Permission completeness check ───────────────────────────────────
// Every child of "Permissions" namespace should have .label and .description

const permissionsObj = resolvePath("Permissions")
if (permissionsObj && typeof permissionsObj === "object") {
  for (const permName of Object.keys(permissionsObj)) {
    const perm = permissionsObj[permName]
    if (typeof perm !== "object" || perm === null) continue
    if (!perm.label) {
      errors.push(
        `[permissions]: missing key "Permissions.${permName}.label"`,
      )
    }
    if (!perm.description) {
      errors.push(
        `[permissions]: missing key "Permissions.${permName}.description"`,
      )
    }
  }
}

// ── Pass 4: Configuration structure check ───────────────────────────────────
// Every direct child of "Configurations" that has sub-objects should have
// .title and .description if it follows the config key pattern

const configurationsObj = resolvePath("Configurations")
if (configurationsObj && typeof configurationsObj === "object") {
  for (const configKey of Object.keys(configurationsObj)) {
    const config = configurationsObj[configKey]
    // Skip non-object children and known special sections
    if (typeof config !== "object" || config === null) continue
    if (configKey === "domainConfigs" || configKey === "notificationEmail") continue

    // Config keys like "timezone", "closing-time" should have .title and .description
    if (!config.title) {
      errors.push(
        `[configurations]: missing key "Configurations.${configKey}.title"`,
      )
    }
    if (!config.description) {
      errors.push(
        `[configurations]: missing key "Configurations.${configKey}.description"`,
      )
    }
  }
}

// ── Report ──────────────────────────────────────────────────────────────────

if (warnings.length > 0) {
  console.log(`\n⚠ ${warnings.length} warning(s) (dynamic keys, skipped):`)
  for (const w of warnings) console.log(`  ${w}`)
}

if (errors.length > 0) {
  console.error(`\n✗ ${errors.length} missing translation key(s):`)
  for (const e of errors) console.error(`  ${e}`)
  process.exit(1)
} else {
  console.log(`\n✓ All statically-extractable translation keys are valid.`)
  process.exit(0)
}
