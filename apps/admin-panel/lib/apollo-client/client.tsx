"use client"

import createUploadLink from "apollo-upload-client/createUploadLink.mjs"

import { relayStylePagination } from "@apollo/client/utilities"

import {
  ApolloClient,
  ApolloNextAppProvider,
  InMemoryCache,
  SSRMultipartLink,
} from "@apollo/experimental-nextjs-app-support"

function makeClient({ coreAdminGqlUrl }: { coreAdminGqlUrl: string }) {
  const uploadLink = createUploadLink({
    uri: coreAdminGqlUrl,
    credentials: "include",
  })
  const ssrMultipartLink = new SSRMultipartLink({
    stripDefer: true,
  })
  const link =
    typeof window === "undefined" ? ssrMultipartLink.concat(uploadLink) : uploadLink

  const cache = new InMemoryCache({
    typePolicies: {
      AccountSetAndSubAccounts: {
        fields: {
          subAccounts: relayStylePagination(),
        },
      },
      Query: {
        fields: {
          customers: relayStylePagination(),
          loans: relayStylePagination(),
          creditFacilities: relayStylePagination(),
        },
      },
    },
  })

  return new ApolloClient({
    cache,
    link,
  })
}

export default function ApolloWrapper({
  config,
  children,
}: {
  config: {
    coreAdminGqlUrl: string
  }
  children: React.ReactNode
}) {
  const client = makeClient(config)
  return (
    <ApolloNextAppProvider makeClient={() => client}>{children}</ApolloNextAppProvider>
  )
}
