"use client"

import React from "react"
import { gql, useApolloClient } from "@apollo/client"

import {
  useRealtimePriceUpdatedSubscription,
  GetRealtimePriceUpdatesDocument,
} from "@/lib/graphql/generated"

gql`
  query GetRealtimePriceUpdates {
    realtimePrice {
      usdCentsPerBtc
    }
  }
`

gql`
  subscription RealtimePriceUpdated {
    realtimePriceUpdated {
      usdCentsPerBtc
    }
  }
`

const RealtimePriceUpdates = () => {
  const client = useApolloClient()

  useRealtimePriceUpdatedSubscription({
    onData: ({ data }) => {
      if (data.data?.realtimePriceUpdated) {
        // Write to cache using the same query document that other components read from
        client.writeQuery({
          query: GetRealtimePriceUpdatesDocument,
          data: {
            realtimePrice: data.data.realtimePriceUpdated,
          },
        })
      }
    },
  })

  return <></>
}

export { RealtimePriceUpdates }
