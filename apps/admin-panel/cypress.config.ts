import { defineConfig } from "cypress"

export default defineConfig({
  e2e: {
    specPattern: [
      "cypress/e2e/user.cy.ts",
      "cypress/e2e/credit-facilities.cy.ts",
      "cypress/e2e/customers.cy.ts",
      "cypress/e2e/transactions.cy.ts",
      "cypress/e2e/terms-templates.cy.ts",
      "cypress/e2e/governance.cy.ts",
      "cypress/e2e/reporting.cy.ts",
      "cypress/e2e/chart-of-accounts.cy.ts",
      "cypress/e2e/trial-balance.cy.ts",
      "cypress/e2e/balance-sheet.cy.ts"
    ],
    baseUrl:
      process.env.BACKEND_ENV === "development"
        ? "http://localhost:4455/admin-panel"
        : "https://admin.staging.lava.galoy.io",
    defaultCommandTimeout: 10000,
    requestTimeout: 10000,
    video: true,
    env: {
      MAGIC_LINK: process.env.MAGIC_LINK,
    },
  },
})
