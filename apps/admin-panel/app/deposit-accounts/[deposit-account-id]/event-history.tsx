"use client"

import React from "react"
import { gql } from "@apollo/client"

import { EntityEventHistory } from "@/components/entity-event-history"
import { DEFAULT_PAGESIZE } from "@/components/paginated-table"
import { useDepositAccountEventHistoryQuery } from "@/lib/graphql/generated"

gql`
  query DepositAccountEventHistory($id: PublicId!, $first: Int!, $after: String) {
    depositAccountByPublicId(id: $id) {
      id
      eventHistory(first: $first, after: $after) {
        ...EventHistoryConnectionFields
      }
    }
  }
`

type DepositAccountEventHistoryProps = {
  depositAccountPublicId: string
}

export const DepositAccountEventHistory: React.FC<
  DepositAccountEventHistoryProps
> = ({ depositAccountPublicId }) => {
  const { data, loading, fetchMore } = useDepositAccountEventHistoryQuery({
    variables: { id: depositAccountPublicId, first: DEFAULT_PAGESIZE },
  })

  return (
    <EntityEventHistory
      translationNamespace="DepositAccounts.DepositAccountDetails.eventHistory"
      data={data?.depositAccountByPublicId?.eventHistory}
      loading={loading}
      fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
    />
  )
}
