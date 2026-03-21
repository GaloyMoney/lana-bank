// Reads *Connection return types from the schema → generates relayStylePagination policies.
/* eslint-disable @typescript-eslint/no-require-imports */
const { isObjectType } = require("graphql")

const PAGINATION_ARGS = new Set(["first", "last", "after", "before"])

function unwrapType(type) {
  return type.ofType ? unwrapType(type.ofType) : type
}

module.exports = {
  plugin(schema) {
    const entityTypes = new Map()
    const queryFields = []

    for (const [typeName, type] of Object.entries(schema.getTypeMap())) {
      if (!isObjectType(type) || typeName.startsWith("__")) continue

      for (const [fieldName, field] of Object.entries(type.getFields())) {
        if (!unwrapType(field.type).name?.endsWith("Connection")) continue

        if (typeName === "Query") {
          const keyArgs = field.args
            .map((a) => a.name)
            .filter((a) => !PAGINATION_ARGS.has(a))
            .sort()
          queryFields.push([fieldName, keyArgs])
        } else {
          if (!entityTypes.has(typeName)) entityTypes.set(typeName, [])
          entityTypes.get(typeName).push(fieldName)
        }
      }
    }

    const lines = [
      "// @generated — do not edit.",
      "/* eslint-disable */",
      'import { relayStylePagination } from "@apollo/client/utilities"',
      "",
      "export const entityPaginationPolicies = {",
    ]

    for (const typeName of [...entityTypes.keys()].sort()) {
      const fields = entityTypes.get(typeName).sort()
      const fieldEntries = fields.map((f) => `${f}: relayStylePagination()`).join(", ")
      lines.push(`  ${typeName}: { fields: { ${fieldEntries} } },`)
    }
    lines.push("}", "")

    queryFields.sort(([a], [b]) => a.localeCompare(b))
    lines.push("export const queryPaginationPolicies = {")
    for (const [f, keyArgs] of queryFields) {
      if (keyArgs.length === 0) {
        lines.push(`  ${f}: relayStylePagination(),`)
      } else {
        lines.push(`  ${f}: { ...relayStylePagination(), keyArgs: [${keyArgs.map((a) => `"${a}"`).join(", ")}] },`)
      }
    }
    lines.push("}", "")

    return lines.join("\n")
  },
}
