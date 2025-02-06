import { Configuration, FrontendApi } from "@ory/client"

import axios from "axios"

import { basePath, env } from "@/env"

export const kratosPublic = () =>
  new FrontendApi(
    new Configuration({
      basePath: env.NEXT_PUBLIC_KRATOS_PUBLIC_API + basePath,
    }),
    "",
    axios,
  )
