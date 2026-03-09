"use client"

import React from "react"
import { gql } from "@apollo/client"

import { EntityEventHistory } from "@/components/entity-event-history"
import { useUserEventHistoryQuery } from "@/lib/graphql/generated"

gql`
  query UserEventHistory($id: UUID!, $first: Int!, $after: String) {
    user(id: $id) {
      id
      eventHistory(first: $first, after: $after) {
        edges {
          cursor
          node {
            eventType
            recordedAt
            sequence
            auditEntryId
            subject {
              ... on User { userId, email }
              ... on System { actor }
            }
            payload
          }
        }
        nodes {
          eventType
          recordedAt
          sequence
          auditEntryId
          subject {
            ... on User { userId, email }
            ... on System { actor }
          }
          payload
        }
        pageInfo {
          hasNextPage
          endCursor
        }
      }
    }
  }
`

type UserEventHistoryProps = {
  userId: string
}

export const UserEventHistory: React.FC<UserEventHistoryProps> = ({ userId }) => {
  const { data, loading } = useUserEventHistoryQuery({
    variables: { id: userId, first: 100 },
  })

  return (
    <EntityEventHistory
      translationNamespace="Users.eventHistory"
      eventTranslationNamespace="EntityEvents.user"
      events={data?.user?.eventHistory.nodes ?? []}
      loading={loading}
    />
  )
}
