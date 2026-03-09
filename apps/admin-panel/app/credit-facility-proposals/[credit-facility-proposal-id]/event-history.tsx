"use client"

import React from "react"
import { gql } from "@apollo/client"

import { EntityEventHistory } from "@/components/entity-event-history"
import { useCreditFacilityProposalEventHistoryQuery } from "@/lib/graphql/generated"

gql`
  query CreditFacilityProposalEventHistory($id: UUID!, $first: Int!, $after: String) {
    creditFacilityProposal(id: $id) {
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

type CreditFacilityProposalEventHistoryProps = {
  proposalId: string
}

export const CreditFacilityProposalEventHistory: React.FC<
  CreditFacilityProposalEventHistoryProps
> = ({ proposalId }) => {
  const { data, loading } = useCreditFacilityProposalEventHistoryQuery({
    variables: { id: proposalId, first: 100 },
  })

  return (
    <EntityEventHistory
      translationNamespace="CreditFacilityProposals.ProposalDetails.eventHistory"
      eventTranslationNamespace="EntityEvents.creditFacilityProposal"
      events={data?.creditFacilityProposal?.eventHistory.nodes ?? []}
      loading={loading}
    />
  )
}
