"use client"

import { gql } from "@apollo/client"

import { useGetCreditFacilityPublicIdQuery } from "@/lib/graphql/generated"

gql`
  query GetCreditFacilityPublicId($id: UUID!) {
    creditFacility(id: $id) {
      publicId
    }
  }
`

export function usePublicIdForCreditFacility(creditFacilityId?: string) {
  const { data, loading, error } = useGetCreditFacilityPublicIdQuery({
    variables: creditFacilityId ? { id: creditFacilityId } : (undefined as never),
    skip: !creditFacilityId,
    fetchPolicy: "cache-and-network",
  })

  return {
    publicId: data?.creditFacility?.publicId ?? null,
    loading,
    error,
  }
}
