"use client"
import { gql } from "@apollo/client"

import WithdrawalDetailsCard from "./details"

import { useGetWithdrawalDetailsQuery } from "@/lib/graphql/generated"
import { DetailsPageSkeleton } from "@/components/details-page-skeleton"

gql`
  query GetWithdrawalDetails($id: UUID!) {
    withdrawal(id: $id) {
      withdrawalId
      amount
      status
      reference

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

function WithdrawalPage({
  params,
}: {
  params: {
    "withdrawal-id": string
  }
}) {
  const { "withdrawal-id": withdrawalId } = params
  const { data, loading, error, refetch } = useGetWithdrawalDetailsQuery({
    variables: { id: withdrawalId },
  })

  if (loading) {
    return <DetailsPageSkeleton tabs={0} tabsCards={0} />
  }
  if (error) return <div className="text-destructive">{error.message}</div>
  if (!data?.withdrawal) return <div>Not found</div>

  return (
    <main className="max-w-7xl m-auto">
      <WithdrawalDetailsCard withdrawal={data.withdrawal} refetch={refetch} />
    </main>
  )
}

export default WithdrawalPage
