import React from "react"
import { unstable_noStore as noStore } from "next/cache"

import ApolloWrapper from "@/lib/apollo-client/client"

import { env } from "@/env"

const config = {
  coreAdminGqlUrl: env.NEXT_PUBLIC_CORE_ADMIN_URL,
}

function ApolloServerWrapper({ children }: { children: React.ReactNode }) {
  noStore()

  return <ApolloWrapper config={config}>{children}</ApolloWrapper>
}

export default ApolloServerWrapper
