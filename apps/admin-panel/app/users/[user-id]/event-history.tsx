"use client"

import React from "react"
import { gql } from "@apollo/client"

import { EntityEventHistory } from "@/components/entity-event-history"
import { DEFAULT_PAGESIZE } from "@/components/paginated-table"
import { useUserEventHistoryQuery } from "@/lib/graphql/generated"

gql`
  query UserEventHistory($id: UUID!, $first: Int!, $after: String) {
    user(id: $id) {
      id
      eventHistory(first: $first, after: $after) {
        ...EventHistoryConnectionFields
      }
    }
  }
`

type UserEventHistoryProps = {
  userId: string
}

export const UserEventHistory: React.FC<UserEventHistoryProps> = ({ userId }) => {
  const { data, loading, fetchMore } = useUserEventHistoryQuery({
    variables: { id: userId, first: DEFAULT_PAGESIZE },
  })

  return (
    <EntityEventHistory
      translationNamespace="Users.eventHistory"
      data={data?.user?.eventHistory}
      loading={loading}
      fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
    />
  )
}
