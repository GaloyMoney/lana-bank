import { defineConfig } from "cypress"

export default defineConfig({
  e2e: {
    baseUrl:
      process.env.NODE_ENV === "development"
        ? "http://localhost:4455/admin-panel"
        : "https://admin.staging.lava.galoy.io",
    defaultCommandTimeout: 10000,
    requestTimeout: 10000,
    video: true,
  },
})
