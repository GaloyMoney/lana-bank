"use client"
import React, { useEffect } from "react"
import { gql } from "@apollo/client"

import { DisbursalDetailsCard } from "./details"

import { VotersCard } from "./voters"

import { DetailsPageSkeleton } from "@/components/details-page-skeleton"
import { useGetDisbursalDetailsQuery } from "@/lib/graphql/generated"
import { useCreateContext } from "@/app/create"

gql`
  query GetDisbursalDetails($id: UUID!) {
    disbursal(id: $id) {
      id
      disbursalId
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
  const { data, loading, error } = useGetDisbursalDetailsQuery({
    variables: { id: disbursalId },
  })
  const { setDisbursal } = useCreateContext()

  useEffect(() => {
    data?.disbursal && setDisbursal(data?.disbursal)
    return () => setDisbursal(null)
  }, [data?.disbursal, setDisbursal])

  if (loading && !data) {
    return <DetailsPageSkeleton tabs={0} detailItems={5} tabsCards={0} />
  }
  if (error) return <div className="text-destructive">{error.message}</div>
  if (!data?.disbursal) return <div>Not found</div>

  return (
    <main className="max-w-7xl m-auto">
      <DisbursalDetailsCard disbursal={data.disbursal} />
      {data.disbursal.approvalProcess && (
        <VotersCard approvalProcess={data.disbursal.approvalProcess} />
      )}
    </main>
  )
}

export default DisbursalPage
