import { Configuration, FrontendApi } from "@ory/client"

import axios from "axios"

import { env as nextRunTimeEnv } from "next-runtime-env"

import { env } from "@/env"

export const kratosPublic = () =>
  new FrontendApi(
    new Configuration({
      basePath:
        nextRunTimeEnv("NEXT_PUBLIC_KRATOS_PUBLIC_API") ||
        env.NEXT_PUBLIC_KRATOS_PUBLIC_API,
    }),
    "",
    axios,
  )
