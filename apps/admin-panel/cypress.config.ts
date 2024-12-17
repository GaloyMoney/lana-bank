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
      "cypress/e2e/balance-sheet.cy.ts",
      "cypress/e2e/dashboard.cy.ts",
      "cypress/e2e/profit-and-loss.cy.ts",
    ],
    baseUrl: "http://localhost:4455/admin-panel",
    defaultCommandTimeout: 10000,
    requestTimeout: 10000,
    video: false,
    screenshotsFolder: "cypress/manuals/screenshots",
    env: {
      MAGIC_LINK: process.env.MAGIC_LINK,
    },
  },
})
