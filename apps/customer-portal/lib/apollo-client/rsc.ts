import { ApolloClient, ApolloLink, HttpLink, InMemoryCache } from "@apollo/client"
import { registerApolloClient } from "@apollo/experimental-nextjs-app-support"
import { headers } from "next/headers"

import { basePath, env } from "@/env"

export const { getClient } = registerApolloClient(() => {
  return new ApolloClient({
    cache: new InMemoryCache(),
    link: ApolloLink.from([
      new HttpLink({
        uri: `${env.NEXT_PUBLIC_CORE_URL + basePath}/graphql`,
        fetchOptions: { cache: "no-store" },
        headers: {
          cookie: headers().get("cookie") ?? "",
        },
      }),
    ]),
  })
})
