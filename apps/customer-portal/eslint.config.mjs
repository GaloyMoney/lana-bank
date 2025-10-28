import { defineConfig } from "eslint/config";
import { fixupConfigRules, fixupPluginRules } from "@eslint/compat";
import typescriptEslint from "@typescript-eslint/eslint-plugin";
import _import from "eslint-plugin-import";
import prettier from "eslint-plugin-prettier";
import path from "node:path";
import { fileURLToPath } from "node:url";
import js from "@eslint/js";
import { FlatCompat } from "@eslint/eslintrc";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const compat = new FlatCompat({
    baseDirectory: __dirname,
    recommendedConfig: js.configs.recommended,
    allConfig: js.configs.all
});

export default defineConfig([{
    extends: fixupConfigRules(compat.extends(
        "next/core-web-vitals",
        "plugin:storybook/recommended",
        "plugin:import/typescript",
        "plugin:@typescript-eslint/recommended",
        "prettier",
    )),

    plugins: {
        "@typescript-eslint": fixupPluginRules(typescriptEslint),
        import: fixupPluginRules(_import),
        prettier,
    },

    rules: {
        "@typescript-eslint/no-extra-semi": "off",
        "@typescript-eslint/no-unused-vars": "error",
        "@typescript-eslint/prefer-for-of": "error",
        "@typescript-eslint/unified-signatures": "error",
        "import/no-deprecated": "error",
        "import/no-extraneous-dependencies": "error",
        "import/no-unassigned-import": "error",
        "import/no-unresolved": "off",

        "import/order": ["error", {
            "newlines-between": "always-and-inside-groups",
        }],

        "arrow-body-style": "off",
        "prefer-arrow-callback": "error",
        "no-duplicate-imports": "error",
        "no-empty-function": "error",

        "no-empty": ["error", {
            allowEmptyCatch: true,
        }],

        "no-new-wrappers": "error",
        "no-param-reassign": "error",
        "no-return-await": "error",
        "no-sequences": "error",
        "no-throw-literal": "error",
        "no-void": "error",
        "@typescript-eslint/explicit-module-boundary-types": "off",
        "no-async-promise-executor": "off",

        "prettier/prettier": ["error", {
            semi: false,
            trailingComma: "all",
            printWidth: 90,
            quoteProps: "consistent",
            singleQuote: false,
            tabWidth: 2,
            useTabs: false,
            bracketSpacing: true,
            arrowParens: "always",
            proseWrap: "preserve",
            endOfLine: "lf",
        }],
    },
}]);