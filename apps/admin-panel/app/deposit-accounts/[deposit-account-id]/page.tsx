"use client"

import { gql } from "@apollo/client"
import { use, useEffect } from "react"
import { useTranslations } from "next-intl"

import DepositAccountDetailsCard from "./details"
import { DepositAccountTransactionsTable } from "./transactions-table"

import { useGetDepositAccountDetailsQuery } from "@/lib/graphql/generated"
import { DetailsPageSkeleton } from "@/components/details-page-skeleton"
import { useBreadcrumb } from "@/app/breadcrumb-provider"
import { PublicIdBadge } from "@/components/public-id-badge"

gql`
  query GetDepositAccountDetails($publicId: PublicId!, $first: Int!, $after: String) {
    depositAccountByPublicId(id: $publicId) {
      id
      publicId
      depositAccountId
      createdAt
      status
      balance {
        settled
        pending
      }
      ledgerAccounts {
        depositAccountId
        frozenDepositAccountId
      }
      customer {
        id
        customerId
        publicId
        applicantId
        email
      }
      history(first: $first, after: $after) {
        pageInfo {
          hasNextPage
          endCursor
          hasPreviousPage
          startCursor
        }
        edges {
          cursor
          node {
            ... on DepositEntry {
              __typename
              recordedAt
              deposit {
                id
                depositId
                publicId
                accountId
                amount
                createdAt
                reference
                status
              }
            }
            ... on WithdrawalEntry {
              __typename
              recordedAt
              withdrawal {
                id
                withdrawalId
                publicId
                accountId
                amount
                createdAt
                reference
                status
              }
            }
            ... on CancelledWithdrawalEntry {
              __typename
              recordedAt
              withdrawal {
                id
                withdrawalId
                publicId
                accountId
                amount
                createdAt
                reference
                status
              }
            }
            ... on DisbursalEntry {
              __typename
              recordedAt
              disbursal {
                id
                disbursalId
                publicId
                amount
                createdAt
                status
              }
            }
            ... on PaymentEntry {
              __typename
              recordedAt
              payment {
                id
                paymentAllocationId
                amount
                createdAt
              }
            }
          }
        }
      }
    }
  }
`

function DepositAccountPage({
  params,
}: {
  params: Promise<{
    "deposit-account-id": string
  }>
}) {
  const { "deposit-account-id": publicId } = use(params)
  const { setCustomLinks, resetToDefault } = useBreadcrumb()
  const navTranslations = useTranslations("Sidebar.navItems")

  const { data, loading, error, fetchMore } = useGetDepositAccountDetailsQuery({
    variables: {
      publicId,
      first: 20,
      after: null,
    },
  })

  useEffect(() => {
    if (data?.depositAccountByPublicId) {
      setCustomLinks([
        { title: navTranslations("depositAccounts"), href: "/deposit-accounts" },
        {
          title: <PublicIdBadge publicId={data.depositAccountByPublicId.publicId} />,
          isCurrentPage: true,
        },
      ])
    }
    return () => {
      resetToDefault()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [data?.depositAccountByPublicId])

  if (loading && !data) {
    return <DetailsPageSkeleton tabs={0} tabsCards={0} />
  }
  if (error) return <div className="text-destructive">{error.message}</div>
  if (!data?.depositAccountByPublicId) return <div>Not found</div>

  return (
    <main className="max-w-7xl m-auto space-y-2">
      <DepositAccountDetailsCard depositAccount={data.depositAccountByPublicId} />
      <DepositAccountTransactionsTable
        history={data.depositAccountByPublicId.history}
        loading={loading}
        fetchMore={fetchMore}
      />
    </main>
  )
}

export default DepositAccountPage
