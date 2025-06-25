import { createEnv } from "@t3-oss/env-nextjs"
import { z } from "zod"

export const env = createEnv({
  shared: {
    NEXT_PUBLIC_CORE_ADMIN_URL: z.string().default("/graphql"),
  },
  client: {
    NEXT_PUBLIC_APP_VERSION: z.string().default("0.0.1-dev"),
  },
  runtimeEnv: {
    NEXT_PUBLIC_CORE_ADMIN_URL: process.env.NEXT_PUBLIC_CORE_ADMIN_URL,
    NEXT_PUBLIC_APP_VERSION: process.env.NEXT_PUBLIC_APP_VERSION,
  },
})
