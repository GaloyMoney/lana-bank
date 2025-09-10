"use client"

import { Resolvers, ApolloClient, InMemoryCache } from "@apollo/client"
import { relayStylePagination } from "@apollo/client/utilities"
import { setContext } from "@apollo/client/link/context"
import createUploadLink from "apollo-upload-client/createUploadLink.mjs"

import { getToken } from "@/app/auth/keycloak"

import {
  CreditFacility,
  CreditFacilityProposal,
  GetRealtimePriceUpdatesDocument,
  GetRealtimePriceUpdatesQuery,
} from "@/lib/graphql/generated"

import { CENTS_PER_USD, SATS_PER_BTC } from "@/lib/utils"

export const makeClient = ({ coreAdminGqlUrl }: { coreAdminGqlUrl: string }) => {
  const uploadLink = createUploadLink({
    uri: coreAdminGqlUrl,
    credentials: "include",
  })

  const authLink = setContext(() => {
    const token = getToken()
    return {
      headers: {
        Authorization: token ? `Bearer ${token}` : "",
      },
    }
  })

  const link = authLink.concat(uploadLink)

  const cache = new InMemoryCache({
    typePolicies: {
      AccountSetAndSubAccounts: {
        fields: {
          subAccounts: relayStylePagination(),
        },
      },
      LedgerAccount: {
        fields: {
          history: relayStylePagination(),
        },
      },
      TrialBalance: {
        fields: {
          accounts: relayStylePagination(),
        },
      },
      Query: {
        fields: {
          customers: { ...relayStylePagination(), keyArgs: ["sort", "filter"] },
          creditFacilities: { ...relayStylePagination(), keyArgs: ["sort", "filter"] },
          creditFacilitiesForStatus: {
            ...relayStylePagination(),
            keyArgs: ["sort", "status"],
          },
          creditFacilitiesForCollateralizationState: {
            ...relayStylePagination(),
            keyArgs: ["sort", "collateralizationState"],
          },
          deposits: relayStylePagination(),
          withdrawals: relayStylePagination(),
          loans: relayStylePagination(),
          committees: relayStylePagination(),
          audit: relayStylePagination(),
          generalLedgerEntries: relayStylePagination(),
          journalEntries: relayStylePagination(),
          transactionTemplates: relayStylePagination(),
          ledgerTransactionsForTemplateCode: relayStylePagination(),
          reportRuns: relayStylePagination(),
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
    CreditFacility: {
      collateralToMatchInitialCvl: async (facility: CreditFacility, _, { cache }) => {
        const priceInfo = await fetchData(cache)
        if (!priceInfo) return null

        const bitcoinPrice = priceInfo.realtimePrice.usdCentsPerBtc / CENTS_PER_USD
        const basisAmountInUsd = facility.facilityAmount / CENTS_PER_USD

        const initialCvlDecimal =
          facility.creditFacilityTerms.initialCvl.__typename === "FiniteCVLPct"
            ? Number(facility.creditFacilityTerms.initialCvl.value || 0) / 100
            : Infinity

        const requiredCollateralInSats =
          (initialCvlDecimal * basisAmountInUsd * SATS_PER_BTC) / bitcoinPrice

        return Math.floor(requiredCollateralInSats)
      },
    },
    CreditFacilityProposal: {
      collateralToMatchInitialCvl: async (
        proposal: CreditFacilityProposal,
        _,
        { cache },
      ) => {
        const priceInfo = await fetchData(cache)
        if (!priceInfo) return null

        const bitcoinPrice = priceInfo.realtimePrice.usdCentsPerBtc / CENTS_PER_USD
        const basisAmountInUsd = proposal.facilityAmount / CENTS_PER_USD

        const initialCvlDecimal =
          proposal.creditFacilityTerms.initialCvl.__typename === "FiniteCVLPct"
            ? Number(proposal.creditFacilityTerms.initialCvl.value || 0) / 100
            : Infinity

        const requiredCollateralInSats =
          (initialCvlDecimal * basisAmountInUsd * SATS_PER_BTC) / bitcoinPrice

        return Math.floor(requiredCollateralInSats)
      },
    },
  }

  return new ApolloClient({
    cache,
    resolvers,
    link,
    defaultOptions: {
      watchQuery: {
        fetchPolicy: "cache-and-network",
      },
    },
  })
}
