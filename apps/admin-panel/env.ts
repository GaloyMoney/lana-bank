import { createEnv } from "@t3-oss/env-nextjs"
import { z } from "zod"

export const env = createEnv({
  shared: {
    NEXT_PUBLIC_CORE_ADMIN_URL: z.string().default("/graphql"),
  },
  client: {
    NEXT_PUBLIC_APP_VERSION: z.string().default("0.0.1-dev"),
    NEXT_PUBLIC_KEYCLOAK_URL: z.string().default("http://localhost:8081"),
    NEXT_PUBLIC_KEYCLOAK_REALM: z.string().default("lana-admin"),
    NEXT_PUBLIC_KEYCLOAK_CLIENT_ID: z.string().default("lana-admin-panel"),
  },
  runtimeEnv: {
    NEXT_PUBLIC_CORE_ADMIN_URL: process.env.NEXT_PUBLIC_CORE_ADMIN_URL,
    NEXT_PUBLIC_APP_VERSION: process.env.NEXT_PUBLIC_APP_VERSION,
    NEXT_PUBLIC_KEYCLOAK_URL: process.env.NEXT_PUBLIC_KEYCLOAK_URL,
    NEXT_PUBLIC_KEYCLOAK_REALM: process.env.NEXT_PUBLIC_KEYCLOAK_REALM,
    NEXT_PUBLIC_KEYCLOAK_CLIENT_ID: process.env.NEXT_PUBLIC_KEYCLOAK_CLIENT_ID,
  },
})
