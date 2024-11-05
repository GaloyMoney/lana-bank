import { Pool } from "pg"

import { env } from "@/env"

export const pool = new Pool({
  connectionString: env.NEXT_AUTH_DATABASE_URL,
})
