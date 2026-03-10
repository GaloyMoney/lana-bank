"use client"

import { gql } from "@apollo/client"
import { use, useState } from "react"

import { FaBan, FaCheckCircle, FaQuestion } from "react-icons/fa"
import { useTranslations } from "next-intl"

import { Tabs, TabsList, TabsTrigger, TabsContent } from "@lana/web/ui/tabs"
import { ScrollArea, ScrollBar } from "@lana/web/ui/scroll-area"
import { LayoutDashboard, CalendarCheck } from "lucide-react"

import { CreditFacilityProposalHeader, CreditFacilityProposalDetailsContent } from "./details"

import { CreditFacilityTermsCard } from "./terms-card"

import { NotFound } from "@/components/not-found"

import { DetailsPageSkeleton } from "@/components/details-page-skeleton"

import {
  useGetCreditFacilityProposalLayoutDetailsQuery,
  CreditFacilityProposalStatus,
  useCreditFacilityProposalConcludedSubscription,
  ApprovalProcessStatus,
  ApprovalProcessFieldsFragment,
} from "@/lib/graphql/generated"

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
    custodian {
      name
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

const ProposalVotersSection = ({
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
      <div className="p-4">
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

export default function CreditFacilityProposalLayout({
  children,
  params,
}: {
  children: React.ReactNode
  params: Promise<{ "credit-facility-proposal-id": string }>
}) {
  const { "credit-facility-proposal-id": proposalId } = use(params)
  const t = useTranslations("CreditFacilityProposals.ProposalDetails.layout")
  const [activeTab, setActiveTab] = useState(OVERVIEW)

  const { data, loading, error } = useGetCreditFacilityProposalLayoutDetailsQuery({
    variables: { creditFacilityProposalId: proposalId },
  })

  useCreditFacilityProposalConcludedSubscription(
    data?.creditFacilityProposal &&
      data.creditFacilityProposal.status ===
        CreditFacilityProposalStatus.PendingApproval
      ? { variables: { creditFacilityProposalId: proposalId } }
      : { skip: true },
  )

  if (loading && !data) return <DetailsPageSkeleton detailItems={4} tabs={2} />
  if (error) return <div className="text-destructive">{error.message}</div>
  if (!data?.creditFacilityProposal) return <NotFound />

  const proposal = data.creditFacilityProposal

  return (
    <main className="max-w-7xl w-full mx-auto border-l border-r flex-1">
      <CreditFacilityProposalHeader proposalDetails={proposal} />
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
          <CreditFacilityProposalDetailsContent proposalDetails={proposal} />
          {proposal.approvalProcess && (
            <>
              <div className="h-1 bg-secondary border-t" />
              <ProposalVotersSection approvalProcess={proposal.approvalProcess} />
            </>
          )}
          <div className="h-1 bg-secondary border-t" />
          <CreditFacilityTermsCard creditFacilityProposal={proposal} />
        </TabsContent>
        <TabsContent value={REPAYMENT_PLAN}>
          {children}
        </TabsContent>
      </Tabs>
    </main>
  )
}
