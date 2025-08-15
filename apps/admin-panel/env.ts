import { createEnv } from "@t3-oss/env-nextjs"
import { z } from "zod"

export const env = createEnv({
  server: {
    KEYCLOAK_URL: z.string().default("http://localhost:8081"),
    KEYCLOAK_REALM: z.string().default("internal"),
    KEYCLOAK_CLIENT_ID: z.string().default("admin-panel"),
  },
  shared: {
    NEXT_PUBLIC_CORE_ADMIN_URL: z.string().default("/graphql"),
  },
  client: {
    NEXT_PUBLIC_APP_VERSION: z.string().default("0.0.1-dev"),
  },
  runtimeEnv: {
    NEXT_PUBLIC_CORE_ADMIN_URL: process.env.NEXT_PUBLIC_CORE_ADMIN_URL,
    NEXT_PUBLIC_APP_VERSION: process.env.NEXT_PUBLIC_APP_VERSION,
    KEYCLOAK_URL: process.env.KEYCLOAK_URL,
    KEYCLOAK_REALM: process.env.KEYCLOAK_REALM,
    KEYCLOAK_CLIENT_ID: process.env.KEYCLOAK_CLIENT_ID,
  },
})
