import { defineConfig } from "cypress"

export default defineConfig({
  e2e: {
    baseUrl: "http://app.localhost:4455",
    screenshotOnRunFailure: false,
  },
})
