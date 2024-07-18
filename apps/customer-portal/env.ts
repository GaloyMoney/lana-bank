import { createEnv } from "@t3-oss/env-nextjs"
import { z } from "zod"

export const env = createEnv({
  server: {
    KRATOS_ADMIN_API: z.string().url().default("http://localhost:4434"),
  },
  shared: {
    NEXT_PUBLIC_CORE_URL: z.string().url().default("http://localhost:4455"),
    NEXT_PUBLIC_KRATOS_PUBLIC_API: z.string().url().default("http://localhost:4455"),
    JWKS_URL: z.string().url().default("http://localhost:4456/.well-known/jwks.json"),
  },
  runtimeEnv: {
    KRATOS_ADMIN_API: process.env.KRATOS_ADMIN_API,

    NEXT_PUBLIC_CORE_URL: process.env.NEXT_PUBLIC_CORE_URL,
    NEXT_PUBLIC_KRATOS_PUBLIC_API: process.env.NEXT_PUBLIC_KRATOS_PUBLIC_API,
    JWKS_URL: process.env.JWKS_URL,
  },
})
