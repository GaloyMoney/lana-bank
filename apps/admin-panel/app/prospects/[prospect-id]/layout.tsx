"use client"

import { gql } from "@apollo/client"
import { use, useEffect } from "react"
import { useTranslations } from "next-intl"

import { ProspectDetailsCard } from "./details"
import { ProspectKycStatus } from "./kyc-status"

import {
  useGetProspectBasicDetailsQuery,
  useProspectKycUpdatedSubscription,
} from "@/lib/graphql/generated"
import { useBreadcrumb } from "@/app/breadcrumb-provider"
import { DetailsPageSkeleton } from "@/components/details-page-skeleton"
import { PublicIdBadge } from "@/components/public-id-badge"

gql`
  fragment ProspectDetailsFragment on Prospect {
    id
    prospectId
    email
    telegramHandle
    stage
    status
    kycStatus
    level
    applicantId
    verificationLink
    verificationLinkCreatedAt
    customerType
    createdAt
    publicId
    personalInfo {
      firstName
      lastName
      dateOfBirth
      nationality
      address
    }
    customer {
      publicId
      email
      customerId
    }
  }

  query GetProspectBasicDetails($id: PublicId!) {
    prospectByPublicId(id: $id) {
      ...ProspectDetailsFragment
    }
  }

  subscription ProspectKycUpdated($prospectId: UUID!) {
    prospectKycUpdated(prospectId: $prospectId) {
      prospect {
        ...ProspectDetailsFragment
      }
    }
  }
`

export default function ProspectLayout({
  children,
  params,
}: {
  children: React.ReactNode
  params: Promise<{ "prospect-id": string }>
}) {
  const t = useTranslations("Prospects.ProspectDetails.layout")
  const navTranslations = useTranslations("Sidebar.navItems")

  const { "prospect-id": prospectId } = use(params)

  const { setCustomLinks, resetToDefault } = useBreadcrumb()

  const { data, loading, error } = useGetProspectBasicDetailsQuery({
    variables: { id: prospectId },
  })

  useProspectKycUpdatedSubscription(
    data?.prospectByPublicId?.prospectId
      ? { variables: { prospectId: data.prospectByPublicId.prospectId } }
      : { skip: true },
  )

  useEffect(() => {
    if (data?.prospectByPublicId) {
      setCustomLinks([
        { title: navTranslations("prospects"), href: "/prospects" },
        {
          title: <PublicIdBadge publicId={data.prospectByPublicId.publicId} />,
          href: `/prospects/${prospectId}`,
        },
      ])
    }
    return () => {
      resetToDefault()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [data?.prospectByPublicId])

  if (loading && !data) return <DetailsPageSkeleton detailItems={3} tabs={1} />
  if (error) return <div className="text-destructive">{t("errors.error")}</div>
  if (!data || !data.prospectByPublicId) return null

  const prospect = data.prospectByPublicId

  return (
    <main className="max-w-7xl m-auto">
      <ProspectDetailsCard prospect={prospect} />
      <div className="flex flex-col md:flex-row w-full gap-2 my-2">
        <ProspectKycStatus
          prospectId={prospect.prospectId}
          kycStatus={prospect.kycStatus}
          level={prospect.level}
          applicantId={prospect.applicantId}
          verificationLink={prospect.verificationLink}
          verificationLinkCreatedAt={prospect.verificationLinkCreatedAt}
        />
      </div>
      {children}
    </main>
  )
}
