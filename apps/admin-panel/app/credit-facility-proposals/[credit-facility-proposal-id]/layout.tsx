"use client"

import { gql } from "@apollo/client"
import { use } from "react"
import { useTranslations } from "next-intl"

import CreditFacilityProposalDetailsCard from "./details"

import { CreditFacilityTermsCard } from "./terms-card"

import { DetailsPageSkeleton } from "@/components/details-page-skeleton"

import {
  useGetCreditFacilityProposalLayoutDetailsQuery,
  CreditFacilityProposalStatus,
  useCreditFacilityProposalConcludedSubscription,
} from "@/lib/graphql/generated"

gql`
  fragment CreditFacilityProposalLayoutFragment on CreditFacilityProposal {
    id
    creditFacilityProposalId
    customerId
    approvalProcessId
    createdAt
    status
    facilityAmount
    custodian {
      name
    }
    creditFacilityTerms {
      annualRate
      accrualInterval
      accrualCycleInterval
      oneTimeFeeRate
      disbursalPolicy
      duration {
        period
        units
      }
      liquidationCvl {
        __typename
        ... on FiniteCvlPct {
          value
        }
        ... on InfiniteCvlPct {
          isInfinite
        }
      }
      marginCallCvl {
        __typename
        ... on FiniteCvlPct {
          value
        }
        ... on InfiniteCvlPct {
          isInfinite
        }
      }
      initialCvl {
        __typename
        ... on FiniteCvlPct {
          value
        }
        ... on InfiniteCvlPct {
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

  subscription creditFacilityProposalConcluded($creditFacilityProposalId: UUID!) {
    creditFacilityProposalConcluded(creditFacilityProposalId: $creditFacilityProposalId) {
      status
      creditFacilityProposal {
        ...CreditFacilityProposalLayoutFragment
      }
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

  useCreditFacilityProposalConcludedSubscription(
    data?.creditFacilityProposal &&
      data?.creditFacilityProposal?.status ===
        CreditFacilityProposalStatus.PendingApproval
      ? { variables: { creditFacilityProposalId: proposalId } }
      : { skip: true },
  )

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
