"use client"

import { gql, useApolloClient } from "@apollo/client"
import { use, useEffect } from "react"
import { useTranslations } from "next-intl"

import { Tabs, TabsList, TabsTrigger, TabsContent } from "@lana/web/ui/tab"

import CreditFacilityDetailsCard from "./details"
import { CreditFacilityCollateral } from "./collateral-card"
import FacilityCard from "./facility-card"

import { DetailsPageSkeleton } from "@/components/details-page-skeleton"
import { useTabNavigation } from "@/hooks/use-tab-navigation"

import {
  ApprovalProcessStatus,
  CreditFacility,
  CreditFacilityStatus,
  GetCreditFacilityLayoutDetailsDocument,
  GetCreditFacilityRepaymentPlanDocument,
  GetCreditFacilityHistoryDocument,
  useGetCreditFacilityLayoutDetailsQuery,
} from "@/lib/graphql/generated"
import { useCreateContext } from "@/app/create"
import { VotersCard } from "@/app/disbursals/[disbursal-id]/voters"

gql`
  fragment CreditFacilityLayoutFragment on CreditFacility {
    id
    creditFacilityId
    status
    facilityAmount
    maturesAt
    collateralizationState
    createdAt
    currentCvl
    publicId
    collateralToMatchInitialCvl @client
    disbursals {
      status
    }
    balance {
      facilityRemaining {
        usdBalance
      }
      disbursed {
        total {
          usdBalance
        }
        outstandingPayable {
          usdBalance
        }
        outstanding {
          usdBalance
        }
      }
      interest {
        total {
          usdBalance
        }
        outstanding {
          usdBalance
        }
      }
      outstanding {
        usdBalance
      }
      collateral {
        btcBalance
      }
    }
    creditFacilityTerms {
      annualRate
      liquidationCvl
      marginCallCvl
      initialCvl
      oneTimeFeeRate
      duration {
        period
        units
      }
    }
    repaymentPlan {
      repaymentType
      status
      initial
      outstanding
      accrualAt
      dueAt
    }
    customer {
      customerId
      publicId
      customerType
      email
    }
    wallet {
      id
      walletId
      address
    }
    approvalProcess {
      id
      deniedReason
      status
      subjectCanSubmitDecision
      approvalProcessId
      approvalProcessType
      createdAt
      ...ApprovalProcessFields
    }
    subjectCanUpdateCollateral
    subjectCanInitiateDisbursal
    subjectCanRecordPayment
    subjectCanComplete
  }

  query GetCreditFacilityLayoutDetails($publicId: PublicId!) {
    creditFacilityByPublicId(id: $publicId) {
      ...CreditFacilityLayoutFragment
    }
  }
`

export default function CreditFacilityLayout({
  children,
  params,
}: {
  children: React.ReactNode
  params: Promise<{ "credit-facility-id": string }>
}) {
  const t = useTranslations("CreditFacilities.CreditFacilityDetails.Layout")

  const { "credit-facility-id": publicId } = use(params)
  const client = useApolloClient()
  const { setFacility } = useCreateContext()

  const TABS = [
    { id: "1", url: "/", tabLabel: t("tabs.history") },
    { id: "4", url: "/disbursals", tabLabel: t("tabs.disbursals") },
    { id: "5", url: "/repayment-plan", tabLabel: t("tabs.repaymentPlan") },
  ]

  const { currentTab, handleTabChange } = useTabNavigation(TABS, publicId)

  const { data, loading, error } = useGetCreditFacilityLayoutDetailsQuery({
    variables: { publicId },
    fetchPolicy: "cache-and-network",
  })

  useEffect(() => {
    data?.creditFacilityByPublicId &&
      setFacility(data?.creditFacilityByPublicId as CreditFacility)
    return () => setFacility(null)
  }, [data?.creditFacilityByPublicId, setFacility])

  useEffect(() => {
    if (
      data?.creditFacilityByPublicId?.status === CreditFacilityStatus.PendingApproval &&
      data?.creditFacilityByPublicId?.approvalProcess?.status ===
        ApprovalProcessStatus.Approved
    ) {
      const timer = setInterval(() => {
        client.query({
          query: GetCreditFacilityLayoutDetailsDocument,
          variables: { publicId },
          fetchPolicy: "network-only",
        })
        client.query({
          query: GetCreditFacilityHistoryDocument,
          variables: { publicId },
          fetchPolicy: "network-only",
        })
        client.query({
          query: GetCreditFacilityRepaymentPlanDocument,
          variables: { publicId },
          fetchPolicy: "network-only",
        })
      }, 3000)

      return () => clearInterval(timer)
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [
    data?.creditFacilityByPublicId?.status,
    data?.creditFacilityByPublicId?.approvalProcess?.status,
  ])

  if (loading && !data) return <DetailsPageSkeleton detailItems={4} tabs={4} />
  if (error) return <div className="text-destructive">{error.message}</div>
  if (!data?.creditFacilityByPublicId) return <div>{t("errors.notFound")}</div>

  return (
    <main className="max-w-7xl m-auto">
      <CreditFacilityDetailsCard
        creditFacilityId={data.creditFacilityByPublicId.creditFacilityId}
        creditFacilityDetails={data.creditFacilityByPublicId}
      />
      <div className="flex md:flex-row flex-col gap-2 my-2">
        <FacilityCard creditFacility={data.creditFacilityByPublicId} />
        <CreditFacilityCollateral creditFacility={data.creditFacilityByPublicId} />
      </div>
      <VotersCard approvalProcess={data.creditFacilityByPublicId.approvalProcess} />
      <Tabs
        defaultValue={TABS[0].url}
        value={currentTab}
        onValueChange={handleTabChange}
        className="mt-2"
      >
        <TabsList>
          {TABS.map((tab) => (
            <TabsTrigger key={tab.url} value={tab.url}>
              {tab.tabLabel}
            </TabsTrigger>
          ))}
        </TabsList>
        {TABS.map((tab) => (
          <TabsContent key={tab.url} value={tab.url}>
            {children}
          </TabsContent>
        ))}
      </Tabs>
    </main>
  )
}
