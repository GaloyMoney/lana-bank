"use client"

import {
  Resolvers,
  ApolloClient,
  InMemoryCache,
  split,
  ApolloLink,
  Observable,
  FetchResult,
  Operation,
} from "@apollo/client"
import { relayStylePagination, getMainDefinition } from "@apollo/client/utilities"
import { setContext } from "@apollo/client/link/context"
import createUploadLink from "apollo-upload-client/createUploadLink.mjs"
import { createClient } from "graphql-sse"
import { print } from "graphql"

import { getToken } from "@/app/auth/keycloak"

import {
  CreditFacility,
  CreditFacilityProposal,
  PendingCreditFacility,
  Cvlpct,
  GetRealtimePriceUpdatesDocument,
  GetRealtimePriceUpdatesQuery,
} from "@/lib/graphql/generated"

import { CENTS_PER_USD, SATS_PER_BTC } from "@/lib/utils"

class SSELink extends ApolloLink {
  private client: ReturnType<typeof createClient>

  constructor(url: string) {
    super()
    this.client = createClient({
      url,
      headers: (): Record<string, string> => {
        const token = getToken()
        return token ? { Authorization: `Bearer ${token}` } : {}
      },
    })
  }

  public request(operation: Operation) {
    return new Observable<FetchResult>((observer) => {
      const { query, variables, operationName } = operation

      const unsubscribe = this.client.subscribe(
        {
          query: print(query),
          variables,
          operationName,
        },
        {
          next: (data) => observer.next?.(data as FetchResult),
          error: (err) => observer.error?.(err),
          complete: () => observer.complete?.(),
        },
      )

      return () => unsubscribe()
    })
  }
}

export const makeClient = ({
  coreAdminGqlUrl,
  coreAdminSseUrl,
}: {
  coreAdminGqlUrl: string
  coreAdminSseUrl: string
}) => {
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

  const httpLink = authLink.concat(uploadLink)
  const sseLink = new SSELink(coreAdminSseUrl)

  const link = split(
    ({ query }) => {
      const definition = getMainDefinition(query)
      return (
        definition.kind === "OperationDefinition" &&
        definition.operation === "subscription"
      )
    },
    sseLink,
    httpLink,
  )

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

  const resolvers: Resolvers = {
    CreditFacility: {
      collateralToMatchInitialCvl: async (facility: CreditFacility, _, { cache }) => {
        return calculateRequiredCollateralInSats(
          cache,
          facility.facilityAmount,
          facility.creditFacilityTerms.initialCvl,
        )
      },
    },
    CreditFacilityProposal: {
      collateralToMatchInitialCvl: async (
        proposal: CreditFacilityProposal,
        _,
        { cache },
      ) => {
        return calculateRequiredCollateralInSats(
          cache,
          proposal.facilityAmount,
          proposal.creditFacilityTerms.initialCvl,
        )
      },
    },
    PendingCreditFacility: {
      collateralToMatchInitialCvl: async (
        pendingFacility: PendingCreditFacility,
        _,
        { cache },
      ) => {
        return calculateRequiredCollateralInSats(
          cache,
          pendingFacility.facilityAmount,
          pendingFacility.creditFacilityTerms.initialCvl,
        )
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

const getRealtimePriceFromCache = (
  cache: InMemoryCache,
): Promise<GetRealtimePriceUpdatesQuery> =>
  new Promise((resolve) => {
    const priceInfo = cache.readQuery({
      query: GetRealtimePriceUpdatesDocument,
    }) as GetRealtimePriceUpdatesQuery
    resolve(priceInfo)
  })

const calculateRequiredCollateralInSats = async (
  cache: InMemoryCache,
  facilityAmount: number,
  initialCvl: Cvlpct,
): Promise<number | null> => {
  const priceInfo = await getRealtimePriceFromCache(cache)
  if (!priceInfo) return null

  const bitcoinPrice = priceInfo.realtimePrice.usdCentsPerBtc / CENTS_PER_USD
  const basisAmountInUsd = facilityAmount / CENTS_PER_USD

  const initialCvlDecimal =
    initialCvl.__typename === "FiniteCVLPct"
      ? Number(initialCvl.value || 0) / 100
      : Infinity

  const requiredCollateralInSats =
    (initialCvlDecimal * basisAmountInUsd * SATS_PER_BTC) / bitcoinPrice

  return Math.floor(requiredCollateralInSats)
}
