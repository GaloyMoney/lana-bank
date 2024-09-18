"use client"

import { ApolloLink, HttpLink, Resolvers } from "@apollo/client"
import { relayStylePagination } from "@apollo/client/utilities"

import {
  ApolloClient,
  ApolloNextAppProvider,
  InMemoryCache,
  SSRMultipartLink,
} from "@apollo/experimental-nextjs-app-support"

import {
  Customer,
  GetRealtimePriceUpdatesDocument,
  GetRealtimePriceUpdatesQuery,
  Loan,
  LoanStatus,
} from "@/lib/graphql/generated"

import { CENTS_PER_USD, SATS_PER_BTC } from "@/lib/utils"

function makeClient({ coreAdminGqlUrl }: { coreAdminGqlUrl: string }) {
  const httpLink = new HttpLink({
    uri: coreAdminGqlUrl,
    fetchOptions: { cache: "no-store" },
  })

  const cache = new InMemoryCache({
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
  })

  const fetchData = (cache: InMemoryCache): Promise<GetRealtimePriceUpdatesQuery> =>
    new Promise((resolve) => {
      const priceInfo = cache.readQuery({
        query: GetRealtimePriceUpdatesDocument,
      }) as GetRealtimePriceUpdatesQuery

      resolve(priceInfo)
    })

  const resolvers: Resolvers = {
    Loan: {
      currentCvl: async (loan: Loan, _, { cache }) => {
        const priceInfo = await fetchData(cache)
        if (!priceInfo) return null

        const principalValueInUsd = loan.facility / CENTS_PER_USD

        const collateralValueInSats = loan.balance.collateral.btcBalance
        const collateralValueInCents =
          (priceInfo.realtimePrice.usdCentsPerBtc * collateralValueInSats) / SATS_PER_BTC
        const collateralValueInUsd = collateralValueInCents / CENTS_PER_USD

        const outstandingAmountInUsd = loan.balance.outstanding.usdBalance / CENTS_PER_USD

        if (collateralValueInUsd == 0 || loan.status === LoanStatus.Closed) return 0

        const newOutstandingAmount =
          outstandingAmountInUsd === 0 ? principalValueInUsd : outstandingAmountInUsd
        const cvl = (collateralValueInUsd / newOutstandingAmount) * CENTS_PER_USD

        return Number(cvl.toFixed(2))
      },
      collateralToMatchInitialCvl: async (loan: Loan, _, { cache }) => {
        const priceInfo = await fetchData(cache)
        if (!priceInfo) return null

        return Math.floor(
          ((loan.loanTerms.initialCvl * loan.facility) /
            priceInfo.realtimePrice.usdCentsPerBtc /
            CENTS_PER_USD) *
            SATS_PER_BTC,
        )
      },
    },
    Customer: {
      transactions: async (customer: Customer) => {
        const deposits = customer.deposits
        const withdrawals = customer.withdrawals

        return [...deposits, ...withdrawals].sort((a, b) => {
          return new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime()
        })
      },
    },
  }

  return new ApolloClient({
    defaultOptions: {
      query: {
        fetchPolicy: "no-cache",
      },
      watchQuery: {
        fetchPolicy: "no-cache",
      },
    },
    cache,
    resolvers,
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
