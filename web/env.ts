import { createEnv } from "@t3-oss/env-nextjs";
import { z } from "zod";

export const env = createEnv({
  server: {
    CORE_URL: z.string().url().default("http://localhost:5252"),
  },
  runtimeEnv: {
    CORE_URL: process.env.CORE_URL,
  },
});
