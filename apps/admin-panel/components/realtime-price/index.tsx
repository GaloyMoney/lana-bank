"use client"

import React from "react"
import { gql } from "@apollo/client"

import { useGetRealtimePriceUpdatesQuery } from "@/lib/graphql/generated"

gql`
  query GetRealtimePriceUpdates {
    realtimePrice {
      usdCentsPerBtc
    }
  }
`

const RealtimePriceUpdates = () => {
  const { data } = useGetRealtimePriceUpdatesQuery({
    fetchPolicy: "network-only",
    pollInterval: 5000,
  })

  if (!data?.realtimePrice.usdCentsPerBtc) console.error("price not available")

  return <></>
}

export { RealtimePriceUpdates }
