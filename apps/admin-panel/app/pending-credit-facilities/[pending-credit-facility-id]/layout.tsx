"use client"

import { gql } from "@apollo/client"
import { use } from "react"
import { useTranslations } from "next-intl"

import PendingCreditFacilityDetailsCard from "./details"
import { PendingCreditFacilityCollateral } from "./collateral-card"

import { PendingCreditFacilityTermsCard } from "./terms-card"

import { DetailsPageSkeleton } from "@/components/details-page-skeleton"

import { useGetPendingCreditFacilityLayoutDetailsQuery } from "@/lib/graphql/generated"

gql`
  fragment PendingCreditFacilityLayoutFragment on PendingCreditFacility {
    id
    pendingCreditFacilityId
    approvalProcessId
    createdAt
    status
    facilityAmount
    collateralizationState
    collateral {
      btcBalance
    }
    collateralToMatchInitialCvl @client
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
    wallet {
      id
      walletId
      address
      network
      custodian {
        name
      }
    }
    approvalProcess {
      ...ApprovalProcessFields
    }
  }

  query GetPendingCreditFacilityLayoutDetails($pendingCreditFacilityId: UUID!) {
    pendingCreditFacility(id: $pendingCreditFacilityId) {
      ...PendingCreditFacilityLayoutFragment
    }
  }
`

export default function PendingCreditFacilityLayout({
  children,
  params,
}: {
  children: React.ReactNode
  params: Promise<{ "pending-credit-facility-id": string }>
}) {
  const { "pending-credit-facility-id": pendingId } = use(params)
  const commonT = useTranslations("Common")

  const { data, loading, error } = useGetPendingCreditFacilityLayoutDetailsQuery({
    variables: { pendingCreditFacilityId: pendingId },
  })

  if (loading && !data) return <DetailsPageSkeleton detailItems={4} tabs={2} />
  if (error) return <div className="text-destructive">{error.message}</div>
  if (!data?.pendingCreditFacility) return <div>{commonT("notFound")}</div>

  return (
    <main className="max-w-7xl m-auto">
      <PendingCreditFacilityDetailsCard pendingDetails={data.pendingCreditFacility} />
      <div className="flex md:flex-row gap-2 my-2 w-full">
        <PendingCreditFacilityTermsCard
          pendingCreditFacility={data.pendingCreditFacility}
        />
        <PendingCreditFacilityCollateral pending={data.pendingCreditFacility} />
      </div>
      {children}
    </main>
  )
}
