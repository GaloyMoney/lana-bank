"use client"

import { gql } from "@apollo/client"
import { use } from "react"
import { useTranslations } from "next-intl"

import PendingCreditFacilityDetailsCard from "./details"
import { PendingCreditFacilityCollateral } from "./collateral-card"

import { PendingCreditFacilityTermsCard } from "./terms-card"

import { DetailsPageSkeleton } from "@/components/details-page-skeleton"

import {
  PendingCreditFacilityStatus,
  useGetPendingCreditFacilityLayoutDetailsQuery,
  usePendingCreditFacilityCollateralizationUpdatedSubscription,
  usePendingCreditFacilityCompletedSubscription,
} from "@/lib/graphql/generated"

gql`
  fragment PendingCreditFacilityLayoutFragment on PendingCreditFacility {
    id
    pendingCreditFacilityId
    customerId
    collateralId
    approvalProcessId
    createdAt
    status
    facilityAmount
    collateralizationState
    collateral {
      btcBalance
    }
    collateralToMatchInitialCvl @client
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

  subscription PendingCreditFacilityCollateralizationUpdated(
    $pendingCreditFacilityId: UUID!
  ) {
    pendingCreditFacilityCollateralizationUpdated(
      pendingCreditFacilityId: $pendingCreditFacilityId
    ) {
      pendingCreditFacility {
        ...PendingCreditFacilityLayoutFragment
      }
    }
  }

  subscription pendingCreditFacilityCompleted($pendingCreditFacilityId: UUID!) {
    pendingCreditFacilityCompleted(pendingCreditFacilityId: $pendingCreditFacilityId) {
      pendingCreditFacility {
        ...PendingCreditFacilityLayoutFragment
      }
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

  usePendingCreditFacilityCollateralizationUpdatedSubscription({
    variables: { pendingCreditFacilityId: pendingId },
  })

  const completed =
    data?.pendingCreditFacility?.status === PendingCreditFacilityStatus.Completed

  usePendingCreditFacilityCompletedSubscription(
    data?.pendingCreditFacility && !completed
      ? { variables: { pendingCreditFacilityId: pendingId } }
      : { skip: true },
  )

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
