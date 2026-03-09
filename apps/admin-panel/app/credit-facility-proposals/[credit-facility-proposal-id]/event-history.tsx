"use client"

import React from "react"
import { gql } from "@apollo/client"

import { EntityEventHistory } from "@/components/entity-event-history"
import { DEFAULT_PAGESIZE } from "@/components/paginated-table"
import { useCreditFacilityProposalEventHistoryQuery } from "@/lib/graphql/generated"

gql`
  query CreditFacilityProposalEventHistory($id: UUID!, $first: Int!, $after: String) {
    creditFacilityProposal(id: $id) {
      id
      eventHistory(first: $first, after: $after) {
        ...EventHistoryConnectionFields
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
  const { data, loading, fetchMore } = useCreditFacilityProposalEventHistoryQuery({
    variables: { id: proposalId, first: DEFAULT_PAGESIZE },
  })

  return (
    <EntityEventHistory
      translationNamespace="CreditFacilityProposals.ProposalDetails.eventHistory"
      data={data?.creditFacilityProposal?.eventHistory}
      loading={loading}
      fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
    />
  )
}
