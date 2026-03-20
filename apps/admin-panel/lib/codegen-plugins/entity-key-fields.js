// Reads @entity_key(field: "X") from the schema → generates keyFields per type.
/* eslint-disable @typescript-eslint/no-require-imports */
const { isObjectType } = require("graphql")

module.exports = {
  plugin(schema) {
    const entries = []

    for (const [name, type] of Object.entries(schema.getTypeMap())) {
      if (!isObjectType(type)) continue
      const dir = type.astNode?.directives?.find((d) => d.name.value === "entity_key")
      if (!dir) continue
      const arg = dir.arguments?.find((a) => a.name.value === "field")
      if (!arg || arg.value.kind !== "StringValue") continue
      entries.push([name, arg.value.value])
    }

    entries.sort(([a], [b]) => a.localeCompare(b))

    const lines = entries.map(([t, f]) => `  ${t}: { keyFields: ["${f}"] },`)

    return [
      "// @generated — do not edit.",
      "/* eslint-disable */",
      "export const entityKeyPolicies = {",
      ...lines,
      "} as const",
      "",
    ].join("\n")
  },
}
