import React from "react"
import { unstable_noStore as noStore } from "next/cache"

import ApolloWrapper from "../wrapper"

import { env } from "@/env"

function ApolloServerWrapper({ children }: { children: React.ReactNode }) {
  noStore()
  const config = {
    coreAdminGqlUrl: env.NEXT_PUBLIC_CORE_ADMIN_URL,
  }
  return <ApolloWrapper config={config}>{children}</ApolloWrapper>
}

export default ApolloServerWrapper
