"use client"

import { gql } from "@apollo/client"
import { use } from "react"
import { useTranslations } from "next-intl"

import CreditFacilityProposalDetailsCard from "./details"

import { CreditFacilityTermsCard } from "./terms-card"

import { DetailsPageSkeleton } from "@/components/details-page-skeleton"

import { useGetCreditFacilityProposalLayoutDetailsQuery } from "@/lib/graphql/generated"

gql`
  fragment CreditFacilityProposalLayoutFragment on CreditFacilityProposal {
    id
    creditFacilityProposalId
    approvalProcessId
    createdAt
    status
    facilityAmount
    customer {
      customerId
      customerType
      publicId
      email
    }
    creditFacilityTerms {
      annualRate
      accrualInterval
      accrualCycleInterval
      oneTimeFeeRate
      duration {
        period
        units
      }
      liquidationCvl {
        __typename
        ... on FiniteCVLPct {
          value
        }
        ... on InfiniteCVLPct {
          isInfinite
        }
      }
      marginCallCvl {
        __typename
        ... on FiniteCVLPct {
          value
        }
        ... on InfiniteCVLPct {
          isInfinite
        }
      }
      initialCvl {
        __typename
        ... on FiniteCVLPct {
          value
        }
        ... on InfiniteCVLPct {
          isInfinite
        }
      }
    }
    approvalProcess {
      ...ApprovalProcessFields
    }
  }

  query GetCreditFacilityProposalLayoutDetails($creditFacilityProposalId: UUID!) {
    creditFacilityProposal(id: $creditFacilityProposalId) {
      ...CreditFacilityProposalLayoutFragment
    }
  }
`

export default function CreditFacilityProposalLayout({
  children,
  params,
}: {
  children: React.ReactNode
  params: Promise<{ "credit-facility-proposal-id": string }>
}) {
  const { "credit-facility-proposal-id": proposalId } = use(params)
  const commonT = useTranslations("Common")

  const { data, loading, error } = useGetCreditFacilityProposalLayoutDetailsQuery({
    variables: { creditFacilityProposalId: proposalId },
  })

  if (loading && !data) return <DetailsPageSkeleton detailItems={4} tabs={2} />
  if (error) return <div className="text-destructive">{error.message}</div>
  if (!data?.creditFacilityProposal) return <div>{commonT("notFound")}</div>

  return (
    <main className="max-w-7xl m-auto">
      <CreditFacilityProposalDetailsCard proposalDetails={data.creditFacilityProposal} />
      <div className="flex md:flex-row gap-2 my-2 w-full">
        <CreditFacilityTermsCard creditFacilityProposal={data.creditFacilityProposal} />
      </div>
      {children}
    </main>
  )
}
