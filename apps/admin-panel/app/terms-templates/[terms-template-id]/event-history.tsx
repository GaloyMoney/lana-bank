"use client"

import React from "react"
import { gql } from "@apollo/client"

import { EntityEventHistory } from "@/components/entity-event-history"
import { DEFAULT_PAGESIZE } from "@/components/paginated-table"
import { useTermsTemplateEventHistoryQuery } from "@/lib/graphql/generated"

gql`
  query TermsTemplateEventHistory($id: UUID!, $first: Int!, $after: String) {
    termsTemplate(id: $id) {
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
        pageInfo {
          hasNextPage
          hasPreviousPage
          startCursor
          endCursor
        }
      }
    }
  }
`

type TermsTemplateEventHistoryProps = {
  termsTemplateId: string
}

export const TermsTemplateEventHistory: React.FC<TermsTemplateEventHistoryProps> = ({
  termsTemplateId,
}) => {
  const { data, loading, fetchMore } = useTermsTemplateEventHistoryQuery({
    variables: { id: termsTemplateId, first: DEFAULT_PAGESIZE },
  })

  return (
    <EntityEventHistory
      translationNamespace="TermsTemplates.TermsTemplateDetails.eventHistory"
      data={data?.termsTemplate?.eventHistory}
      loading={loading}
      fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
    />
  )
}
