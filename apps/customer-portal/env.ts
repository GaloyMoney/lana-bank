import { createEnv } from "@t3-oss/env-nextjs"
import { z } from "zod"

export const env = createEnv({
  server: {
    AUTH_SECRET: z.string().default("secret"),
    AUTH_KEYCLOAK_SECRET: z.string().default("41jv8RepY6r797jIyucj30OiZsFJXCls"),
  },
  shared: {
    NEXT_PUBLIC_CORE_URL: z.string().default("http://app.localhost:4455"),
    NEXT_PUBLIC_KEYCLOAK_URL: z.string().default("http://localhost:8081"),
    NEXT_PUBLIC_KEYCLOAK_REALM: z.string().default("customer"),
    NEXT_PUBLIC_KEYCLOAK_CLIENT_ID: z.string().default("customer-portal"),
  },
  runtimeEnv: {
    AUTH_SECRET: process.env.AUTH_SECRET,
    AUTH_KEYCLOAK_SECRET: process.env.AUTH_KEYCLOAK_SECRET,
    NEXT_PUBLIC_CORE_URL: process.env.NEXT_PUBLIC_CORE_URL,
    NEXT_PUBLIC_KEYCLOAK_URL: process.env.NEXT_PUBLIC_KEYCLOAK_URL,
    NEXT_PUBLIC_KEYCLOAK_REALM: process.env.NEXT_PUBLIC_KEYCLOAK_REALM,
    NEXT_PUBLIC_KEYCLOAK_CLIENT_ID: process.env.NEXT_PUBLIC_KEYCLOAK_CLIENT_ID,
  },
})
