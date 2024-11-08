"use client"
import React from "react"
import { gql } from "@apollo/client"

import { DisbursalDetailsCard } from "./details"

import { VotersCard } from "./voters"

import { BreadcrumbLink, BreadCrumbWrapper } from "@/components/breadcrumb-wrapper"
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
          email
          customerId
        }
      }
      approvalProcess {
        approvalProcessId
        deniedReason
        approvalProcessType
        createdAt
        subjectCanSubmitDecision
        status
        rules {
          ... on CommitteeThreshold {
            threshold
            committee {
              name
              currentMembers {
                email
                roles
              }
            }
          }
          ... on SystemApproval {
            autoApprove
          }
        }
        voters {
          stillEligible
          didVote
          didApprove
          didDeny
          user {
            userId
            email
            roles
          }
        }
      }
    }
  }
`

const DisbursalBreadcrumb = ({ disbursalId }: { disbursalId: string }) => {
  const links: BreadcrumbLink[] = [
    { title: "Dashboard", href: "/dashboard" },
    { title: "Disbursals", href: "/disbursals" },
    { title: `Disbursal ${disbursalId}`, isCurrentPage: true },
  ]

  return <BreadCrumbWrapper links={links} />
}

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
      <DisbursalBreadcrumb disbursalId={data.disbursal.disbursalId} />
      <DisbursalDetailsCard disbursal={data.disbursal} refetch={refetch} />
      {data.disbursal.approvalProcess && (
        <VotersCard approvalProcess={data.disbursal.approvalProcess} />
      )}
    </main>
  )
}

export default DisbursalPage
