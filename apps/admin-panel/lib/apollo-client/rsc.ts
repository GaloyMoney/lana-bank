import { ApolloClient, HttpLink, InMemoryCache } from "@apollo/client"
import { registerApolloClient } from "@apollo/experimental-nextjs-app-support"
import { headers } from "next/headers"

import { env } from "@/env"

export const { getClient } = registerApolloClient(() => {
  return new ApolloClient({
    cache: new InMemoryCache(),
    link: new HttpLink({
      uri: env.NEXT_PUBLIC_CORE_ADMIN_URL,
      fetchOptions: { cache: "no-store" },
      headers: {
        /* Next Auth authentication - imitating user on server for SSR */
        cookie: headers().get("cookie") || "",
      },
    }),
  })
})
