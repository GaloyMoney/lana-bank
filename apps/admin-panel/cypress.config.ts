import { defineConfig } from "cypress"

const multiplier = 10

export default defineConfig({
  e2e: {
    setupNodeEvents(on) {
      on("task", {
        checkUrl(url: string) {
          return new Promise((resolve) => {
            fetch(url)
              .then((response) => resolve(response.ok))
              .catch(() => resolve(false))
          })
        },
      })
      on("before:browser:launch", (browser, launchOptions) => {
        if (browser.name === "chrome") {
          launchOptions.args.push("--window-size=1920,1080")
          launchOptions.args.push("--disable-dev-shm-usage")
          launchOptions.args.push("--force-device-scale-factor=1")
          return launchOptions
        }

        if (browser.name === "electron") {
          launchOptions.preferences = {
            width: 1920,
            height: 1080,
            frame: false,
            useContentSize: true,
          }
          return launchOptions
        }

        if (browser.name === "firefox") {
          launchOptions.args.push("--width=1920")
          launchOptions.args.push("--height=1080")
          return launchOptions
        }

        return launchOptions
      })
    },
    viewportWidth: 1280,
    viewportHeight: 720,
    specPattern: [
      "cypress/e2e/customers.cy.ts",
      "cypress/e2e/credit-facilities.cy.ts",
      "cypress/e2e/modules.cy.ts",
      "cypress/e2e/user.cy.ts",
      "cypress/e2e/transactions.cy.ts",
      "cypress/e2e/terms-templates.cy.ts",
      "cypress/e2e/governance.cy.ts",
      "cypress/e2e/reporting.cy.ts",
      "cypress/e2e/chart-of-accounts.cy.ts",
      "cypress/e2e/trial-balance.cy.ts",
      "cypress/e2e/balance-sheet.cy.ts",
      "cypress/e2e/dashboard.cy.ts",
      "cypress/e2e/profit-and-loss.cy.ts",
      "cypress/e2e/deposit-accounts.cy.ts",
    ],
    baseUrl: "http://admin.localhost:4455",
    defaultCommandTimeout: 4000 * multiplier,
    requestTimeout: 5000 * multiplier,
    pageLoadTimeout: 60000 * multiplier,
    retries: 5,
    screenshotOnRunFailure: false,
    experimentalMemoryManagement: true,
    video: true,
    screenshotsFolder: "cypress/manuals/screenshots",
    env: {
      COOKIES: process.env.COOKIES,
      TEST_LANGUAGE: "es",
      GOOGLE_CLOUD_AVAILABLE:
        process.env.GOOGLE_APPLICATION_CREDENTIALS &&
        process.env.GOOGLE_APPLICATION_CREDENTIALS.trim() !== ""
          ? true
          : false,
    },
  },
})
