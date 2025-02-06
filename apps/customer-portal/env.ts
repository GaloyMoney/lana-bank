import { createEnv } from "@t3-oss/env-nextjs"
import { z } from "zod"

export const env = createEnv({
  shared: {
    NEXT_PUBLIC_CORE_URL: z.string().url().default("http://localhost:4455"),
    NEXT_PUBLIC_KRATOS_PUBLIC_API: z.string().url().default("http://localhost:4455"),
    NEXT_PUBLIC_BASE_PATH: z.string().default("/"),
  },
  runtimeEnv: {
    NEXT_PUBLIC_CORE_URL: process.env.NEXT_PUBLIC_CORE_URL,
    NEXT_PUBLIC_KRATOS_PUBLIC_API: process.env.NEXT_PUBLIC_KRATOS_PUBLIC_API,
    NEXT_PUBLIC_BASE_PATH: process.env.NEXT_PUBLIC_BASE_PATH,
  },
})

export const basePath = env.NEXT_PUBLIC_BASE_PATH === "/" ? "" : env.NEXT_PUBLIC_BASE_PATH
