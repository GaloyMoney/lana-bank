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
import { useBreadcrumb } from "@/app/breadcrumb-provider"
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
    currentCvl {
      total
      disbursed
    }
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
    customer {
      customerId
      email
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

  query GetCreditFacilityLayoutDetails($id: UUID!) {
    creditFacility(id: $id) {
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
  const navTranslations = useTranslations("Sidebar.navItems")

  const { "credit-facility-id": creditFacilityId } = use(params)
  const { setCustomLinks, resetToDefault } = useBreadcrumb()
  const client = useApolloClient()
  const { setFacility } = useCreateContext()

  const TABS = [
    { id: "1", url: "/", tabLabel: t("tabs.history") },
    { id: "4", url: "/disbursals", tabLabel: t("tabs.disbursals") },
    { id: "5", url: "/repayment-plan", tabLabel: t("tabs.repaymentPlan") },
  ]

  const { currentTab, handleTabChange } = useTabNavigation(TABS, creditFacilityId)

  const { data, loading, error } = useGetCreditFacilityLayoutDetailsQuery({
    variables: { id: creditFacilityId },
    fetchPolicy: "cache-and-network",
  })

  useEffect(() => {
    data?.creditFacility && setFacility(data?.creditFacility as CreditFacility)
    return () => setFacility(null)
  }, [data?.creditFacility, setFacility])

  useEffect(() => {
    if (
      data?.creditFacility?.status === CreditFacilityStatus.PendingApproval &&
      data?.creditFacility?.approvalProcess?.status === ApprovalProcessStatus.Approved
    ) {
      const timer = setInterval(() => {
        client.query({
          query: GetCreditFacilityLayoutDetailsDocument,
          variables: { id: creditFacilityId },
          fetchPolicy: "network-only",
        })
        client.query({
          query: GetCreditFacilityHistoryDocument,
          variables: { id: creditFacilityId },
          fetchPolicy: "network-only",
        })
        client.query({
          query: GetCreditFacilityRepaymentPlanDocument,
          variables: { id: creditFacilityId },
          fetchPolicy: "network-only",
        })
      }, 3000)

      return () => clearInterval(timer)
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [data?.creditFacility?.status, data?.creditFacility?.approvalProcess?.status])

  useEffect(() => {
    if (data?.creditFacility) {
      const currentTabData = TABS.find((tab) => tab.url === currentTab)
      setCustomLinks([
        {
          title: navTranslations("creditFacilities"),
          href: "/credit-facilities",
        },
        {
          title: data.creditFacility.creditFacilityId,
          href: `/credit-facilities/${creditFacilityId}`,
        },
        ...(currentTabData?.url === "/"
          ? []
          : [{ title: currentTabData?.tabLabel ?? "", isCurrentPage: true as const }]),
      ])
    }
    return () => {
      resetToDefault()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [data?.creditFacility, currentTab])

  if (loading && !data) return <DetailsPageSkeleton detailItems={4} tabs={4} />
  if (error) return <div className="text-destructive">{error.message}</div>
  if (!data?.creditFacility) return <div>{t("errors.notFound")}</div>

  return (
    <main className="max-w-7xl m-auto">
      <CreditFacilityDetailsCard
        creditFacilityId={creditFacilityId}
        creditFacilityDetails={data.creditFacility}
      />
      <div className="flex md:flex-row flex-col gap-2 my-2">
        <FacilityCard creditFacility={data.creditFacility} />
        <CreditFacilityCollateral creditFacility={data.creditFacility} />
      </div>
      <VotersCard approvalProcess={data.creditFacility.approvalProcess} />
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
