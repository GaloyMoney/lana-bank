"use client"

import { ApolloLink, HttpLink } from "@apollo/client"
import { relayStylePagination } from "@apollo/client/utilities"
import {
  ApolloClient,
  ApolloNextAppProvider,
  InMemoryCache,
  SSRMultipartLink,
} from "@apollo/experimental-nextjs-app-support"

import {
  GetRealtimePriceUpdatesDocument,
  GetRealtimePriceUpdatesQuery,
  Loan,
  LoanStatus,
} from "@/lib/graphql/generated"

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
        currentCvl: async (loan: Loan, _, { cache }) => {
          const fetchData = () =>
            new Promise((resolve) => {
              const priceInfo = cache.readQuery({
                query: GetRealtimePriceUpdatesDocument,
              }) as GetRealtimePriceUpdatesQuery

              if (priceInfo) {
                resolve(priceInfo)
              } else {
                setTimeout(() => resolve(fetchData()), 500) // Try again after 500 ms
              }
            })

          const priceInfo = (await fetchData()) as GetRealtimePriceUpdatesQuery

          const principalValueInUsd = loan.principal / 100

          const collateralValueInSats = loan.balance.collateral.btcBalance
          const collateralValueInCents =
            (priceInfo.realtimePrice.usdCentsPerBtc * collateralValueInSats) / 100_000_000
          const collateralValueInUsd = collateralValueInCents / 100

          const outstandingAmountInUsd = loan.balance.outstanding.usdBalance / 100

          if (collateralValueInUsd == 0 || loan.status === LoanStatus.Closed) return 0

          const newOutstandingAmount =
            outstandingAmountInUsd === 0 ? principalValueInUsd : outstandingAmountInUsd
          const cvl = (collateralValueInUsd / newOutstandingAmount) * 100

          return Number(cvl.toFixed(2))
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
