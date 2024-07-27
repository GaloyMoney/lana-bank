import { createEnv } from "@t3-oss/env-nextjs"
import { z } from "zod"

export const env = createEnv({
  shared: {
    NEXT_PUBLIC_CORE_ADMIN_URL: z
      .string()
      .url()
      .default("http://localhost:4455/admin/graphql"),

    // NEXTAUTH_* are just here for documentation. they need to be injected in .env for NEXTAUTH to pick them up
    NEXTAUTH_SECRET: z.string().min(8).default("nextAuthSecret"),
    NEXTAUTH_URL: z.string().url().default("http://localhost:4455/admin-panel/api/auth"),
    NEXTAUTH_INTERNAL_URL: z
      .string()
      .url()
      .default("http://localhost:4455/admin-panel/api/auth"),
  },
  server: {
    EMAIL_FROM: z.string().default("no-reply@lava-bank.com"),
    EMAIL_SERVER: z.string().default("smtp://localhost:1025"),
    NEXT_AUTH_DATABASE_URL: z
      .string()
      .url()
      .default("postgres://dbuser:secret@localhost:5435/default?sslmode=disable"),
  },
  runtimeEnv: {
    NEXT_PUBLIC_CORE_ADMIN_URL: process.env.NEXT_PUBLIC_CORE_ADMIN_URL,
    NEXTAUTH_SECRET: process.env.NEXTAUTH_SECRET,
    NEXTAUTH_URL: process.env.NEXTAUTH_URL,
    NEXTAUTH_INTERNAL_URL: process.env.NEXTAUTH_INTERNAL_URL,
    NEXT_AUTH_DATABASE_URL: process.env.NEXT_AUTH_DATABASE_URL,
    EMAIL_SERVER: process.env.EMAIL_SERVER,
    EMAIL_FROM: process.env.EMAIL_FROM,
  },
})
