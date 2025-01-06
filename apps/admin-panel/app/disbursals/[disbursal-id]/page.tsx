"use client"
import React from "react"
import { gql } from "@apollo/client"

import { DisbursalDetailsCard } from "./details"

import { VotersCard } from "./voters"

import { DetailsPageSkeleton } from "@/components/details-page-skeleton"
import { useGetDisbursalDetailsQuery } from "@/lib/graphql/generated"

gql`
  query GetDisbursalDetails($id: UUID!) {
    disbursal(id: $id) {
      id
      disbursalId
      index
      amount
      createdAt
      status
      creditFacility {
        id
        creditFacilityId
        facilityAmount
        status
        customer {
          id
          email
          customerId
          depositAccount {
            balance {
              settled
              pending
            }
          }
        }
      }
      approvalProcess {
        ...ApprovalProcessFields
      }
    }
  }
`

function DisbursalPage({
  params,
}: {
  params: {
    "disbursal-id": string
  }
}) {
  const { "disbursal-id": disbursalId } = params
  const { data, loading, error, refetch } = useGetDisbursalDetailsQuery({
    variables: { id: disbursalId },
  })

  if (loading) {
    return <DetailsPageSkeleton tabs={0} detailItems={5} tabsCards={0} />
  }
  if (error) return <div className="text-destructive">{error.message}</div>
  if (!data?.disbursal) return <div>Not found</div>

  return (
    <main className="max-w-7xl m-auto">
      <DisbursalDetailsCard disbursal={data.disbursal} refetch={refetch} />
      {data.disbursal.approvalProcess && (
        <VotersCard approvalProcess={data.disbursal.approvalProcess} />
      )}
    </main>
  )
}

export default DisbursalPage
