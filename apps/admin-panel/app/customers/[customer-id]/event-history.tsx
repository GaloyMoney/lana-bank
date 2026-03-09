"use client"

import React from "react"
import { gql } from "@apollo/client"

import { EntityEventHistory } from "@/components/entity-event-history"
import { DEFAULT_PAGESIZE } from "@/components/paginated-table"
import { useCustomerEventHistoryQuery } from "@/lib/graphql/generated"

gql`
  query CustomerEventHistory($id: PublicId!, $first: Int!, $after: String) {
    customerByPublicId(id: $id) {
      id
      eventHistory(first: $first, after: $after) {
        ...EventHistoryConnectionFields
      }
    }
  }
`

type CustomerEventHistoryProps = {
  customerId: string
}

export const CustomerEventHistory: React.FC<CustomerEventHistoryProps> = ({
  customerId,
}) => {
  const { data, loading, fetchMore } = useCustomerEventHistoryQuery({
    variables: { id: customerId, first: DEFAULT_PAGESIZE },
  })

  return (
    <EntityEventHistory
      translationNamespace="Customers.CustomerDetails.eventHistory"
      data={data?.customerByPublicId?.eventHistory}
      loading={loading}
      fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
    />
  )
}
