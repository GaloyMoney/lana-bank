import { defineConfig, globalIgnores } from "eslint/config"
import { fixupConfigRules } from "@eslint/compat"
import nextVitals from "eslint-config-next/core-web-vitals"
import nextTs from "eslint-config-next/typescript"
import storybook from "eslint-plugin-storybook"

import prettierConfig from "eslint-config-prettier/flat"

export default defineConfig([
  ...fixupConfigRules(nextVitals),
  ...fixupConfigRules(nextTs),
  ...storybook.configs["flat/recommended"],
  globalIgnores([".next/**", "out/**", "build/**", "next-env.d.ts", "**/generated/**"]),
  {
    rules: {
      "@typescript-eslint/no-extra-semi": "off",
      "@typescript-eslint/no-unused-vars": "error",
      "@typescript-eslint/prefer-for-of": "error",
      "@typescript-eslint/unified-signatures": "error",
      "@typescript-eslint/no-unused-expressions": [
        "error",
        {
          allowTaggedTemplates: true,
          allowShortCircuit: true,
        },
      ],
      "import/no-deprecated": "error",
      "import/no-extraneous-dependencies": "error",
      "import/no-unassigned-import": [
        "error",
        {
          allow: ["**/*.css"],
        },
      ],
      "import/no-unresolved": "off",
      "import/order": ["error", { "newlines-between": "always-and-inside-groups" }],
      "arrow-body-style": "off",
      "prefer-arrow-callback": "error",
      "no-duplicate-imports": "error",
      "no-empty-function": "error",
      "no-empty": ["error", { allowEmptyCatch: true }],
      "no-new-wrappers": "error",
      "no-param-reassign": "error",
      "no-return-await": "error",
      "no-sequences": "error",
      "no-throw-literal": "error",
      "no-void": "error",
      "@typescript-eslint/explicit-module-boundary-types": "off",
      "no-async-promise-executor": "off",
      "react-hooks/set-state-in-effect": "off",
    },
  },
  {
    files: ["cypress/**/*.ts", "cypress/**/*.tsx"],
    rules: {
      "@typescript-eslint/no-unused-expressions": "off",
    },
  },
  prettierConfig,
])
