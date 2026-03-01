"use client"

import { gql } from "@apollo/client"
import { use, useEffect } from "react"
import { useTranslations } from "next-intl"

import { Tabs, TabsList, TabsTrigger, TabsContent } from "@lana/web/ui/tabs"

import CreditFacilityDetailsCard from "./details"
import { CreditFacilityCollateral } from "./collateral-card"
import FacilityCard from "./facility-card"

import { DetailsPageSkeleton } from "@/components/details-page-skeleton"
import { useTabNavigation } from "@/hooks/use-tab-navigation"
import { useBreadcrumb } from "@/app/breadcrumb-provider"
import { PublicIdBadge } from "@/components/public-id-badge"

import {
  CreditFacility,
  useGetCreditFacilityLayoutDetailsQuery,
  useCreditFacilityCollateralizationUpdatedSubscription,
} from "@/lib/graphql/generated"
import { useCreateContext } from "@/app/create"

gql`
  fragment CreditFacilityLayoutFragment on CreditFacility {
    id
    creditFacilityId
    collateralId
    status
    facilityAmount
    maturesAt
    collateralizationState
    activatedAt
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
      oneTimeFeeRate
      disbursalPolicy
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
    wallet {
      id
      walletId
      address
      network
      custodian {
        name
      }
    }
    userCanUpdateCollateral
    userCanInitiateDisbursal
    userCanRecordPayment
    userCanRecordPaymentWithDate
    userCanComplete
  }

  query GetCreditFacilityLayoutDetails($publicId: PublicId!) {
    creditFacilityByPublicId(id: $publicId) {
      ...CreditFacilityLayoutFragment
    }
  }

  subscription creditFacilityCollateralizationUpdated($creditFacilityId: UUID!) {
    creditFacilityCollateralizationUpdated(creditFacilityId: $creditFacilityId) {
      creditFacility {
        ...CreditFacilityLayoutFragment
      }
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

  const { "credit-facility-id": publicId } = use(params)
  const { setFacility } = useCreateContext()
  const { setCustomLinks, resetToDefault } = useBreadcrumb()

  const TABS = [
    { id: "1", url: "/", tabLabel: t("tabs.history") },
    { id: "4", url: "/disbursals", tabLabel: t("tabs.disbursals") },
    { id: "7", url: "/liquidations", tabLabel: t("tabs.liquidations") },
    { id: "5", url: "/repayment-plan", tabLabel: t("tabs.repaymentPlan") },
    { id: "6", url: "/ledger-accounts", tabLabel: t("tabs.ledgerAccounts") },
  ]

  const { currentTab, handleTabChange } = useTabNavigation(TABS, publicId)

  const { data, loading, error } = useGetCreditFacilityLayoutDetailsQuery({
    variables: { publicId },
    fetchPolicy: "cache-and-network",
  })

  const creditFacilityId = data?.creditFacilityByPublicId?.creditFacilityId
  useCreditFacilityCollateralizationUpdatedSubscription(
    creditFacilityId ? { variables: { creditFacilityId } } : { skip: true },
  )

  useEffect(() => {
    data?.creditFacilityByPublicId &&
      setFacility(data?.creditFacilityByPublicId as CreditFacility)
    return () => setFacility(null)
  }, [data?.creditFacilityByPublicId, setFacility])

  useEffect(() => {
    if (data?.creditFacilityByPublicId) {
      const currentTabData = TABS.find((tab) => tab.url === currentTab)
      setCustomLinks([
        { title: navTranslations("creditFacilities"), href: "/credit-facilities" },
        {
          title: <PublicIdBadge publicId={data.creditFacilityByPublicId.publicId} />,
          href: `/credit-facilities/${publicId}`,
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
  }, [data?.creditFacilityByPublicId, currentTab])

  if (loading && !data) return <DetailsPageSkeleton detailItems={4} tabs={4} />
  if (error) return <div className="text-destructive">{error.message}</div>
  if (!data?.creditFacilityByPublicId) return <div>{t("errors.notFound")}</div>

  return (
    <main className="max-w-7xl m-auto">
      <CreditFacilityDetailsCard
        creditFacilityDetails={data.creditFacilityByPublicId}
      />
      <div className="flex md:flex-row flex-col gap-2 my-2">
        <FacilityCard creditFacility={data.creditFacilityByPublicId} />
        <CreditFacilityCollateral creditFacility={data.creditFacilityByPublicId} />
      </div>
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
