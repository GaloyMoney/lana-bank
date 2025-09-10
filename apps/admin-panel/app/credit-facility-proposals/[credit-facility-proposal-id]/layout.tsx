"use client"

import { gql } from "@apollo/client"
import { use } from "react"
import { useTranslations } from "next-intl"

import { Tabs, TabsList, TabsTrigger, TabsContent } from "@lana/web/ui/tab"

import CreditFacilityProposalDetailsCard from "./details"
import { CreditFacilityProposalCollateral } from "./collateral-card"

import { CreditFacilityTermsCard } from "./terms-card"

import { DetailsPageSkeleton } from "@/components/details-page-skeleton"
import { useTabNavigation } from "@/hooks/use-tab-navigation"

import {
  CreditFacilityProposal,
  useGetCreditFacilityProposalLayoutDetailsQuery,
  useGetApprovalProcessByIdQuery,
} from "@/lib/graphql/generated"

gql`
  fragment CreditFacilityProposalLayoutFragment on CreditFacilityProposal {
    id
    creditFacilityProposalId
    approvalProcessId
    createdAt
    status
    facilityAmount
    collateralizationState
    collateral {
      btcBalance
    }
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
    collateralToMatchInitialCvl @client
  }

  query GetCreditFacilityProposalLayoutDetails($creditFacilityProposalId: UUID!) {
    creditFacilityProposal(id: $creditFacilityProposalId) {
      ...CreditFacilityProposalLayoutFragment
    }
  }
`

gql`
  query GetApprovalProcessById($id: UUID!) {
    approvalProcess(id: $id) {
      ...ApprovalProcessFields
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
  const t = useTranslations("CreditFacilityProposals.ProposalDetails.Layout")

  const { data, loading, error } = useGetCreditFacilityProposalLayoutDetailsQuery({
    variables: { creditFacilityProposalId: proposalId },
  })
  const approvalProcessId = data?.creditFacilityProposal?.approvalProcessId
  const { data: approvalData } = useGetApprovalProcessByIdQuery({
    variables: approvalProcessId ? { id: approvalProcessId } : (undefined as never),
    skip: !approvalProcessId,
    fetchPolicy: "cache-and-network",
  })

  const tabs = [
    { id: "1", url: "/", tabLabel: t("tabs.history") },
    { id: "2", url: "/repayment-plan", tabLabel: t("tabs.repaymentPlan") },
  ]

  const { currentTab, handleTabChange } = useTabNavigation(tabs, proposalId)

  if (loading && !data) return <DetailsPageSkeleton detailItems={4} tabs={2} />
  if (error) return <div className="text-destructive">{error.message}</div>
  if (!data?.creditFacilityProposal) return <div>{t("errors.notFound")}</div>

  return (
    <main className="max-w-7xl m-auto">
      <CreditFacilityProposalDetailsCard
        proposalDetails={data.creditFacilityProposal as CreditFacilityProposal}
        approvalProcess={approvalData?.approvalProcess ?? null}
      />
      <div className="flex md:flex-row gap-2 my-2 w-full">
        <CreditFacilityTermsCard creditFacilityProposal={data.creditFacilityProposal} />
        <CreditFacilityProposalCollateral proposal={data.creditFacilityProposal} />
      </div>
      <Tabs
        defaultValue={tabs[0].url}
        value={currentTab}
        onValueChange={handleTabChange}
        className="mt-2"
      >
        <TabsList>
          {tabs.map((tab) => (
            <TabsTrigger key={tab.url} value={tab.url}>
              {tab.tabLabel}
            </TabsTrigger>
          ))}
        </TabsList>
        {tabs.map((tab) => (
          <TabsContent key={tab.url} value={tab.url}>
            {children}
          </TabsContent>
        ))}
      </Tabs>
    </main>
  )
}
