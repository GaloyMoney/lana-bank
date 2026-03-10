"use client"

import { gql } from "@apollo/client"
import { use, useEffect, useState } from "react"

import { FaBan, FaCheckCircle, FaQuestion } from "react-icons/fa"
import { useTranslations } from "next-intl"

import { Tabs, TabsList, TabsTrigger, TabsContent } from "@lana/web/ui/tabs"
import { ScrollArea, ScrollBar } from "@lana/web/ui/scroll-area"
import { LayoutDashboard, CalendarCheck } from "lucide-react"

import { PendingCreditFacilityHeader, PendingCreditFacilityDetailsContent } from "./details"
import { PendingCreditFacilityCollateral } from "./collateral-card"
import { PendingCreditFacilityTermsCard } from "./terms-card"

import { NotFound } from "@/components/not-found"
import { DetailsPageSkeleton } from "@/components/details-page-skeleton"

import {
  PendingCreditFacilityStatus,
  useGetPendingCreditFacilityLayoutDetailsQuery,
  usePendingCreditFacilityCollateralizationUpdatedSubscription,
  usePendingCreditFacilityCompletedSubscription,
  ApprovalProcessStatus,
  ApprovalProcessFieldsFragment,
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

const PendingVotersSection = ({
  approvalProcess,
}: {
  approvalProcess: ApprovalProcessFieldsFragment | null
}) => {
  const t = useTranslations("Disbursals.DisbursalDetails.VotersCard")

  if (!approvalProcess) return null
  if (approvalProcess.rules.__typename !== "CommitteeApproval") return null

  const voters = approvalProcess.voters.filter((voter) => {
    if (
      approvalProcess.status === ApprovalProcessStatus.InProgress ||
      ([ApprovalProcessStatus.Approved, ApprovalProcessStatus.Denied].includes(
        approvalProcess.status as ApprovalProcessStatus,
      ) &&
        voter.didVote)
    ) {
      return true
    }
    return false
  })

  return (
    <div>
      <div className="flex items-center gap-2 px-4 py-2 border-b">
        <h2 className="text-lg font-semibold">
          {t("title", { committeeName: approvalProcess.rules.committee?.name })}
        </h2>
      </div>
      <div className="p-4 border-b">
        <p className="text-sm text-muted-foreground mb-3">{t("description")}</p>
        {voters.map((voter) => (
          <div key={voter.user.userId} className="flex items-center space-x-3 p-2">
            {voter.didApprove ? (
              <FaCheckCircle className="h-6 w-6 text-green-500" />
            ) : voter.didDeny ? (
              <FaBan className="h-6 w-6 text-red-500" />
            ) : !voter.didVote ? (
              <FaQuestion className="h-6 w-6 text-textColor-secondary" />
            ) : (
              <>{/* Impossible */}</>
            )}
            <div>
              <p className="text-sm font-medium">{voter.user.email}</p>
              <p className="text-sm text-textColor-secondary">{voter.user.role?.name}</p>
              <p className="text-xs text-textColor-secondary">
                {voter.didApprove && t("voter.approved")}
                {voter.didDeny && t("voter.denied")}
                {!voter.didVote && t("voter.notVoted")}
              </p>
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}

const OVERVIEW = "overview"
const REPAYMENT_PLAN = "repayment-plan"

export default function PendingCreditFacilityLayout({
  children,
  params,
}: {
  children: React.ReactNode
  params: Promise<{ "pending-credit-facility-id": string }>
}) {
  const { "pending-credit-facility-id": pendingId } = use(params)
  const t = useTranslations("PendingCreditFacilities.PendingDetails.layout")
  const [activeTab, setActiveTab] = useState(OVERVIEW)

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

  const pending = data.pendingCreditFacility

  return (
    <main className="max-w-7xl w-full mx-auto border-l border-r flex-1">
      <PendingCreditFacilityHeader pendingDetails={pending} />
      <Tabs value={activeTab} onValueChange={setActiveTab} className="gap-0">
        <ScrollArea>
          <TabsList className="bg-transparent rounded-none h-auto w-full justify-start p-0 border-b">
            {[
              { value: OVERVIEW, label: t("tabs.overview"), icon: <LayoutDashboard className="h-4 w-4" /> },
              { value: REPAYMENT_PLAN, label: t("tabs.repaymentPlan"), icon: <CalendarCheck className="h-4 w-4" /> },
            ].map((tab) => (
              <TabsTrigger
                key={tab.value}
                value={tab.value}
                className="flex-initial rounded-none border-b-2 border-transparent data-[state=active]:border-b-primary data-[state=active]:bg-transparent data-[state=active]:shadow-none px-4 py-2.5 text-sm gap-1.5"
              >
                {tab.icon}
                {tab.label}
              </TabsTrigger>
            ))}
          </TabsList>
          <ScrollBar orientation="horizontal" />
        </ScrollArea>
        <TabsContent value={OVERVIEW}>
          <PendingCreditFacilityDetailsContent pendingDetails={pending} />
          {pending.approvalProcess && (
            <>
              <div className="h-1 bg-secondary border-t" />
              <PendingVotersSection approvalProcess={pending.approvalProcess} />
            </>
          )}
          <div className="h-1 bg-secondary border-t" />
          <div className="flex flex-col md:flex-row w-full border-b">
            <div className="md:w-[55%] md:border-r">
              <PendingCreditFacilityTermsCard pendingCreditFacility={pending} />
            </div>
            <div className="hidden md:block w-1 bg-secondary border-r" />
            <div className="md:flex-1">
              <PendingCreditFacilityCollateral pending={pending} />
            </div>
          </div>
        </TabsContent>
        <TabsContent value={REPAYMENT_PLAN}>
          {children}
        </TabsContent>
      </Tabs>
    </main>
  )
}
