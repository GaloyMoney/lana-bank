"use client"
import React, { useEffect, use } from "react"
import { gql } from "@apollo/client"

import { DisbursalDetailsCard } from "./details"

import { VotersCard } from "./voters"

import { DetailsPageSkeleton } from "@/components/details-page-skeleton"
import { useGetDisbursalDetailsQuery } from "@/lib/graphql/generated"
import { useCreateContext } from "@/app/create"

gql`
  query GetDisbursalDetails($publicId: PublicId!) {
    disbursalByPublicId(id: $publicId) {
      id
      disbursalId
      amount
      createdAt
      status
      publicId
      creditFacility {
        id
        creditFacilityId
        facilityAmount
        status
        publicId
        customer {
          id
          email
          customerId
          publicId
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
  params: Promise<{
    "disbursal-id": string
  }>
}) {
  const { "disbursal-id": publicId } = use(params)
  const { data, loading, error } = useGetDisbursalDetailsQuery({
    variables: { publicId },
  })
  const { setDisbursal } = useCreateContext()

  useEffect(() => {
    data?.disbursalByPublicId && setDisbursal(data.disbursalByPublicId)
    return () => setDisbursal(null)
  }, [data?.disbursalByPublicId, setDisbursal])

  if (loading && !data) {
    return <DetailsPageSkeleton tabs={0} detailItems={5} tabsCards={0} />
  }
  if (error) return <div className="text-destructive">{error.message}</div>
  if (!data?.disbursalByPublicId) return <div>Not found</div>

  return (
    <main className="max-w-7xl m-auto">
      <DisbursalDetailsCard disbursal={data.disbursalByPublicId} />
      {data.disbursalByPublicId.approvalProcess && (
        <VotersCard approvalProcess={data.disbursalByPublicId.approvalProcess} />
      )}
    </main>
  )
}

export default DisbursalPage
