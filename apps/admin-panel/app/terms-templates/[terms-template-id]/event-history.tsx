"use client"

import React from "react"
import { gql } from "@apollo/client"

import { EntityEventHistory } from "@/components/entity-event-history"
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

type TermsTemplateEventHistoryProps = {
  termsTemplateId: string
}

export const TermsTemplateEventHistory: React.FC<TermsTemplateEventHistoryProps> = ({
  termsTemplateId,
}) => {
  const { data, loading } = useTermsTemplateEventHistoryQuery({
    variables: { id: termsTemplateId, first: 100 },
  })

  return (
    <EntityEventHistory
      translationNamespace="TermsTemplates.TermsTemplateDetails.eventHistory"
      eventTranslationNamespace="EntityEvents.termsTemplate"
      events={data?.termsTemplate?.eventHistory.nodes ?? []}
      loading={loading}
    />
  )
}
