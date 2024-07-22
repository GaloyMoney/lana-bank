import { ApolloClient, ApolloLink, HttpLink, InMemoryCache } from "@apollo/client"
import { registerApolloClient } from "@apollo/experimental-nextjs-app-support"
import { headers } from "next/headers"

import { env } from "@/env"

const customRequest = async (
  uri: RequestInfo | URL,
  options?: RequestInit,
): Promise<Response> => {
  console.log(`Request to ${uri}`)
  console.log(`Request method: ${options?.method}`)
  console.log(`Request headers: ${JSON.stringify(options?.headers)}`)
  if (options?.body) {
    console.log(`Request body: ${options.body}`)
  }

  try {
    const response = await fetch(uri, options)
    console.log(`Response status: ${response.status}`)
    console.log(
      `Response headers: ${JSON.stringify(Object.fromEntries(response.headers.entries()))}`,
    )

    return response
  } catch (error) {
    console.error("Fetch error:", error)
    throw error
  }
}

export const { getClient } = registerApolloClient(() => {
  console.log("nextjs headers start -----")
  console.log(JSON.stringify(headers(), null, 2))
  console.log("nextjs headers end -----")

  const requestHeaders = Object.fromEntries(
    Array.from(headers()).map(([key, value]) => [key, value]),
  )

  return new ApolloClient({
    cache: new InMemoryCache(),
    link: ApolloLink.from([
      new HttpLink({
        uri: `${env.NEXT_PUBLIC_CORE_URL}/graphql`,
        fetchOptions: { cache: "no-store" },
        fetch: customRequest,
        headers: requestHeaders,
      }),
    ]),
  })
})
