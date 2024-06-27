import { Configuration, FrontendApi } from "@ory/client"

import { env } from "@/env"

export const kratosPublic = new FrontendApi(
  new Configuration({ basePath: env.NEXT_PUBLIC_KRATOS_PUBLIC_API }),
)
