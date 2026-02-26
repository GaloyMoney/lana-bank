"use client"

import { gql } from "@apollo/client"
import { use, useEffect } from "react"
import { useTranslations } from "next-intl"

import DepositDetailsCard from "./details"

import { useGetDepositDetailsQuery } from "@/lib/graphql/generated"
import { DetailsPageSkeleton } from "@/components/details-page-skeleton"
import { useBreadcrumb } from "@/app/breadcrumb-provider"
import { PublicIdBadge } from "@/components/public-id-badge"

gql`
  fragment DepositDetailsPageFragment on Deposit {
    id
    depositId
    publicId
    amount
    createdAt
    reference
    status
    account {
      id
      publicId
      customerId
    }
  }

  query GetDepositDetails($publicId: PublicId!) {
    depositByPublicId(id: $publicId) {
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
  const { "deposit-id": publicId } = use(params)
  const { setCustomLinks, resetToDefault } = useBreadcrumb()
  const navTranslations = useTranslations("Sidebar.navItems")

  const { data, loading, error } = useGetDepositDetailsQuery({
    variables: { publicId },
  })

  useEffect(() => {
    if (data?.depositByPublicId) {
      setCustomLinks([
        { title: navTranslations("deposits"), href: "/deposits" },
        {
          title: <PublicIdBadge publicId={data.depositByPublicId.publicId} />,
          isCurrentPage: true,
        },
      ])
    }
    return () => {
      resetToDefault()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [data?.depositByPublicId])

  if (loading && !data) {
    return <DetailsPageSkeleton tabs={0} tabsCards={0} />
  }
  if (error) return <div className="text-destructive">{error.message}</div>
  if (!data?.depositByPublicId) return <div>Not found</div>

  return (
    <main className="max-w-7xl m-auto space-y-2">
      <DepositDetailsCard deposit={data.depositByPublicId} />
    </main>
  )
}

export default DepositPage
