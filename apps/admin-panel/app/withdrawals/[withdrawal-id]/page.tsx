"use client"
import { gql } from "@apollo/client"

import { useEffect, use } from "react"
import { useTranslations } from "next-intl"

import WithdrawalDetailsCard from "./details"

import { useGetWithdrawalDetailsQuery } from "@/lib/graphql/generated"
import { DetailsPageSkeleton } from "@/components/details-page-skeleton"
import { useCreateContext } from "@/app/create"
import { useBreadcrumb } from "@/app/breadcrumb-provider"
import { PublicIdBadge } from "@/components/public-id-badge"

gql`
  fragment WithdrawDetailsPageFragment on Withdrawal {
    id
    withdrawalId
    publicId
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

  query GetWithdrawalDetails($publicId: PublicId!) {
    withdrawalByPublicId(id: $publicId) {
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
  const { "withdrawal-id": publicId } = use(params)
  const { setWithdraw } = useCreateContext()
  const { setCustomLinks, resetToDefault } = useBreadcrumb()
  const navTranslations = useTranslations("Sidebar.navItems")

  const { data, loading, error } = useGetWithdrawalDetailsQuery({
    variables: { publicId },
  })

  useEffect(() => {
    data?.withdrawalByPublicId && setWithdraw(data?.withdrawalByPublicId)
    return () => setWithdraw(null)
  }, [data?.withdrawalByPublicId, setWithdraw])

  useEffect(() => {
    if (data?.withdrawalByPublicId) {
      setCustomLinks([
        { title: navTranslations("withdrawals"), href: "/withdrawals" },
        {
          title: <PublicIdBadge publicId={data.withdrawalByPublicId.publicId} />,
          isCurrentPage: true,
        },
      ])
    }
    return () => {
      resetToDefault()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [data?.withdrawalByPublicId])

  if (loading && !data) {
    return <DetailsPageSkeleton tabs={0} tabsCards={0} />
  }
  if (error) return <div className="text-destructive">{error.message}</div>
  if (!data?.withdrawalByPublicId) return <div>Not found</div>

  return (
    <main className="max-w-7xl m-auto">
      <WithdrawalDetailsCard withdrawal={data.withdrawalByPublicId} />
    </main>
  )
}

export default WithdrawalPage
