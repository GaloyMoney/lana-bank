"use client"

import React from "react"
import { gql } from "@apollo/client"

import { EntityEventHistory } from "@/components/entity-event-history"
import { DEFAULT_PAGESIZE } from "@/components/paginated-table"
import { useDepositEventHistoryQuery } from "@/lib/graphql/generated"

gql`
  query DepositEventHistory($id: PublicId!, $first: Int!, $after: String) {
    depositByPublicId(id: $id) {
      id
      eventHistory(first: $first, after: $after) {
        ...EventHistoryConnectionFields
      }
    }
  }
`

type DepositEventHistoryProps = {
  depositPublicId: string
}

export const DepositEventHistory: React.FC<DepositEventHistoryProps> = ({
  depositPublicId,
}) => {
  const { data, loading, fetchMore } = useDepositEventHistoryQuery({
    variables: { id: depositPublicId, first: DEFAULT_PAGESIZE },
  })

  return (
    <EntityEventHistory
      translationNamespace="Deposits.DepositDetails.eventHistory"
      data={data?.depositByPublicId?.eventHistory}
      loading={loading}
      fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
    />
  )
}
