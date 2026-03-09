"use client"

import { gql } from "@apollo/client"
import { use, useEffect } from "react"
import { useTranslations } from "next-intl"

import { Tabs, TabsList, TabsTrigger, TabsContent } from "@lana/web/ui/tabs"

import PendingCreditFacilityDetailsCard from "./details"
import { PendingCreditFacilityCollateral } from "./collateral-card"
import { PendingCreditFacilityEventHistory } from "./event-history"

import { PendingCreditFacilityTermsCard } from "./terms-card"

import { NotFound } from "@/components/not-found"

import { DetailsPageSkeleton } from "@/components/details-page-skeleton"

import {
  PendingCreditFacilityStatus,
  useGetPendingCreditFacilityLayoutDetailsQuery,
  usePendingCreditFacilityCollateralizationUpdatedSubscription,
  usePendingCreditFacilityCompletedSubscription,
} from "@/lib/graphql/generated"

const PENDING_CREDIT_FACILITY_POLL_INTERVAL_MS = 60_000

gql`
  fragment PendingCreditFacilityLayoutFragment on PendingCreditFacility {
    id
    pendingCreditFacilityId
    creditFacilityId
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
      effectiveAnnualRate
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
  const tTabs = useTranslations("PendingCreditFacilities.PendingDetails.tabs")

  const { data, loading, error, startPolling, stopPolling } =
    useGetPendingCreditFacilityLayoutDetailsQuery({
      variables: { pendingCreditFacilityId: pendingId },
    })

  usePendingCreditFacilityCollateralizationUpdatedSubscription({
    variables: { pendingCreditFacilityId: pendingId },
  })

  const completed =
    data?.pendingCreditFacility?.status === PendingCreditFacilityStatus.Completed

  useEffect(() => {
    if (!data?.pendingCreditFacility || completed) {
      stopPolling()
      return
    }

    startPolling(PENDING_CREDIT_FACILITY_POLL_INTERVAL_MS)

    return () => stopPolling()
  }, [completed, data?.pendingCreditFacility, startPolling, stopPolling])

  usePendingCreditFacilityCompletedSubscription(
    data?.pendingCreditFacility && !completed
      ? { variables: { pendingCreditFacilityId: pendingId } }
      : { skip: true },
  )

  if (loading && !data) return <DetailsPageSkeleton detailItems={4} tabs={2} />
  if (error) return <div className="text-destructive">{error.message}</div>
  if (!data?.pendingCreditFacility) return <NotFound />

  return (
    <main className="max-w-7xl m-auto">
      <PendingCreditFacilityDetailsCard pendingDetails={data.pendingCreditFacility} />
      <div className="flex md:flex-row gap-2 my-2 w-full">
        <PendingCreditFacilityTermsCard
          pendingCreditFacility={data.pendingCreditFacility}
        />
        <PendingCreditFacilityCollateral pending={data.pendingCreditFacility} />
      </div>
      <Tabs defaultValue="repaymentPlan">
        <TabsList>
          <TabsTrigger value="repaymentPlan">{tTabs("repaymentPlan")}</TabsTrigger>
          <TabsTrigger value="events">{tTabs("events")}</TabsTrigger>
        </TabsList>
        <TabsContent value="repaymentPlan">
          {children}
        </TabsContent>
        <TabsContent value="events">
          <PendingCreditFacilityEventHistory pendingId={pendingId} />
        </TabsContent>
      </Tabs>
    </main>
  )
}
