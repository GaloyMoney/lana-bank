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
  const { error }= useGetRealtimePriceUpdatesQuery({
    fetchPolicy: "network-only",
    pollInterval: 5000,
  })

   if (error) {
    console.error("Failed to fetch realtime price updates:", error)
  }

  return null
}

export { RealtimePriceUpdates }
