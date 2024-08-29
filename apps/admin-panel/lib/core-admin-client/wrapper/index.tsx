"use client"

import { ApolloLink, HttpLink } from "@apollo/client"
import { relayStylePagination } from "@apollo/client/utilities"
import {
  ApolloClient,
  ApolloNextAppProvider,
  InMemoryCache,
  SSRMultipartLink,
} from "@apollo/experimental-nextjs-app-support"

import { Loan } from "@/lib/graphql/generated"

function makeClient({ coreAdminGqlUrl }: { coreAdminGqlUrl: string }) {
  const httpLink = new HttpLink({
    uri: coreAdminGqlUrl,
    fetchOptions: { cache: "no-store" },
  })

  return new ApolloClient({
    defaultOptions: {
      query: {
        fetchPolicy: "no-cache",
      },
      watchQuery: {
        fetchPolicy: "no-cache",
      },
    },
    cache: new InMemoryCache({
      typePolicies: {
        AccountSetAndSubAccounts: {
          fields: {
            subAccounts: relayStylePagination(),
          },
        },
        Query: {
          fields: {
            loans: relayStylePagination(),
          },
        },
      },
    }),
    resolvers: {
      Loan: {
        currentCvl: (loan: Loan, _, { cache }) => {
          console.log(loan, cache)
          return 100
        },
      },
    },
    link:
      typeof window === "undefined"
        ? ApolloLink.from([
            new SSRMultipartLink({
              stripDefer: true,
            }),
            httpLink,
          ])
        : httpLink,
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
