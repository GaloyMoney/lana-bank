import { createEnv } from "@t3-oss/env-nextjs"
import { z } from "zod"

export const env = createEnv({
  server: {
    AUTH_SECRET: z.string().default("secret"),
    AUTH_KEYCLOAK_ID: z.string().default("lana-customer"),
    AUTH_KEYCLOAK_SECRET: z.string().default("41jv8RepY6r797jIyucj30OiZsFJXCls"),
    AUTH_KEYCLOAK_ISSUER: z
      .string()
      .default("http://localhost:8080/realms/lana-customer"),
  },
  shared: {
    NEXT_PUBLIC_CORE_URL: z.string().default("http://app.localhost:4455"),
    NEXT_PUBLIC_KEYCLOAK_LOGOUT_URL: z
      .string()
      .default(
        "http://localhost:8081/realms/lana-customer/protocol/openid-connect/logout",
      ),
  },
  runtimeEnv: {
    AUTH_SECRET: process.env.AUTH_SECRET,
    AUTH_KEYCLOAK_ID: process.env.AUTH_KEYCLOAK_ID,
    AUTH_KEYCLOAK_SECRET: process.env.AUTH_KEYCLOAK_SECRET,
    AUTH_KEYCLOAK_ISSUER: process.env.AUTH_KEYCLOAK_ISSUER,
    NEXT_PUBLIC_CORE_URL: process.env.NEXT_PUBLIC_CORE_URL,
    NEXT_PUBLIC_KEYCLOAK_LOGOUT_URL: process.env.NEXT_PUBLIC_KEYCLOAK_LOGOUT_URL,
  },
})
