"use client"
import { gql } from "@apollo/client"

import WithdrawalDetailsCard from "./details"

import { BreadcrumbLink, BreadCrumbWrapper } from "@/components/breadcrumb-wrapper"
import { useGetWithdrawalDetailsQuery } from "@/lib/graphql/generated"
import { DetailsPageSkeleton } from "@/components/details-page-skeleton"

gql`
  query GetWithdrawalDetails($id: UUID!) {
    withdrawal(id: $id) {
      customerId
      withdrawalId
      amount
      status
      reference
      subjectCanConfirm
      subjectCanCancel
      customer {
        email
        customerId
        applicantId
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

const WithdrawalBreadcrumb = ({ withdrawalId }: { withdrawalId: string }) => {
  const links: BreadcrumbLink[] = [
    { title: "Dashboard", href: "/dashboard" },
    { title: "Withdrawals", href: "/withdrawals" },
    { title: `Withdrawal ${withdrawalId}`, isCurrentPage: true },
  ]

  return <BreadCrumbWrapper links={links} />
}

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
      <WithdrawalBreadcrumb withdrawalId={data.withdrawal.withdrawalId} />
      <WithdrawalDetailsCard withdrawal={data.withdrawal} refetch={refetch} />
    </main>
  )
}

export default WithdrawalPage
