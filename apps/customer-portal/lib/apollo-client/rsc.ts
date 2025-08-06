import { ApolloClient, ApolloLink, HttpLink, InMemoryCache } from "@apollo/client"
import { registerApolloClient } from "@apollo/client-integration-nextjs"
import { headers } from "next/headers"

import { auth } from "@/auth"

import { env } from "@/env"

export const { getClient } = registerApolloClient(async () => {
  const headersObj = await headers()
  const session = await auth()

  const requestHeaders = Object.fromEntries(
    Array.from(headersObj).map(([key, value]) => [key, value]),
  )

  if (session?.accessToken) {
    requestHeaders["Authorization"] = `Bearer ${session.accessToken}`
  }

  return new ApolloClient({
    cache: new InMemoryCache(),
    link: ApolloLink.from([
      new HttpLink({
        uri: `${env.NEXT_PUBLIC_CORE_URL}/graphql`,
        fetchOptions: { cache: "no-store" },
        headers: requestHeaders,
      }),
    ]),
  })
})
