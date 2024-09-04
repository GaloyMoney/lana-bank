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
import { CENTS_PER_USD, SATS_PER_BTC } from "../utils"

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

      const principalValueInUsd = loan.principal / CENTS_PER_USD

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
        (loan.loanTerms.initialCvl * loan.principal) /
        priceInfo.realtimePrice.usdCentsPerBtc /
        CENTS_PER_USD * SATS_PER_BTC
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
