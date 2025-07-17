"use client"
import { gql } from "@apollo/client"

import { useEffect, use } from "react"

import WithdrawalDetailsCard from "./details"

import { useGetWithdrawalDetailsQuery } from "@/lib/graphql/generated"
import { DetailsPageSkeleton } from "@/components/details-page-skeleton"
import { useCreateContext } from "@/app/create"

gql`
  fragment WithdrawDetailsPageFragment on Withdrawal {
    id
    withdrawalId
    amount
    status
    reference
    account {
      customer {
        id
        customerId
        publicId
        applicantId
        email
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

  query GetWithdrawalDetails($id: UUID!) {
    withdrawal(id: $id) {
      ...WithdrawDetailsPageFragment
    }
  }
`

function WithdrawalPage({
  params,
}: {
  params: Promise<{
    "withdrawal-id": string
  }>
}) {
  const { "withdrawal-id": withdrawalId } = use(params)
  const { setWithdraw } = useCreateContext()

  const { data, loading, error } = useGetWithdrawalDetailsQuery({
    variables: { id: withdrawalId },
  })

  useEffect(() => {
    data?.withdrawal && setWithdraw(data?.withdrawal)
    return () => setWithdraw(null)
  }, [data?.withdrawal, setWithdraw])

  if (loading && !data) {
    return <DetailsPageSkeleton tabs={0} tabsCards={0} />
  }
  if (error) return <div className="text-destructive">{error.message}</div>
  if (!data?.withdrawal) return <div>Not found</div>

  return (
    <main className="max-w-7xl m-auto">
      <WithdrawalDetailsCard withdrawal={data.withdrawal} />
    </main>
  )
}

export default WithdrawalPage
