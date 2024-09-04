"use client"

import { ApolloLink, HttpLink, Resolvers } from "@apollo/client"
import { relayStylePagination } from "@apollo/client/utilities"

import {
  ApolloClient,
  InMemoryCache,
  SSRMultipartLink,
} from "@apollo/experimental-nextjs-app-support"

import {
  GetRealtimePriceUpdatesDocument,
  GetRealtimePriceUpdatesQuery,
  Loan,
  LoanStatus,
} from "@/lib/graphql/generated"

import { env } from "@/env";

const httpLink = new HttpLink({
  uri: env.NEXT_PUBLIC_CORE_ADMIN_URL,
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

const fetchData = (cache: any): Promise<GetRealtimePriceUpdatesQuery> =>
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
    collateralToMatchInitialCvl: async (loan: Loan, _, { cache }) => {
      const priceInfo = await fetchData(cache)
      if (!priceInfo) return null

      return (
        (loan.loanTerms.initialCvl * loan.principal) /
        priceInfo.realtimePrice.usdCentsPerBtc /
        100
      )
    },
  },
}

const client = new ApolloClient({
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

export const makeClient = () => client
