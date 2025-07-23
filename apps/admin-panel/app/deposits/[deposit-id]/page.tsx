"use client"

import { gql } from "@apollo/client"
import { use } from "react"

import DepositDetailsCard from "./details"

import { useGetDepositDetailsQuery } from "@/lib/graphql/generated"
import { DetailsPageSkeleton } from "@/components/details-page-skeleton"

gql`
  fragment DepositDetailsPageFragment on Deposit {
    id
    depositId
    amount
    createdAt
    reference
    status
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
  }

  query GetDepositDetails($id: UUID!) {
    deposit(id: $id) {
      ...DepositDetailsPageFragment
    }
  }
`

function DepositPage({
  params,
}: {
  params: Promise<{
    "deposit-id": string
  }>
}) {
  const { "deposit-id": depositId } = use(params)

  const { data, loading, error } = useGetDepositDetailsQuery({
    variables: { id: depositId },
  })

  if (loading && !data) {
    return <DetailsPageSkeleton tabs={0} tabsCards={0} />
  }
  if (error) return <div className="text-destructive">{error.message}</div>
  if (!data?.deposit) return <div>Not found</div>

  return (
    <main className="max-w-7xl m-auto">
      <DepositDetailsCard deposit={data.deposit} />
    </main>
  )
}

export default DepositPage
