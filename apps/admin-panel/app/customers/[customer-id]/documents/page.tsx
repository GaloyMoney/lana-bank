"use client"

import { gql } from "@apollo/client"
import { use } from "react"

import { Documents } from "./list"

import { useGetCustomerDocumentsQuery } from "@/lib/graphql/generated"

gql`
  query GetCustomerDocuments($id: PublicId!) {
    customerByPublicId(id: $id) {
      id
      customerId
      documents {
        id
        filename
        documentId
      }
    }
  }
`

export default function CustomerDocumentsPage({
  params,
}: {
  params: Promise<{ "customer-id": string }>
}) {
  const { "customer-id": customerId } = use(params)
  const { data, refetch } = useGetCustomerDocumentsQuery({
    variables: { id: customerId },
  })

  if (!data?.customerByPublicId) return null

  return <Documents customer={data.customerByPublicId} refetch={refetch} />
}
