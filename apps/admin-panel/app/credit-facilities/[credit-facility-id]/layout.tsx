"use client"

import { gql } from "@apollo/client"
import { use, useEffect, useState } from "react"
import { useTranslations } from "next-intl"

import { Tabs, TabsList, TabsTrigger, TabsContent } from "@lana/web/ui/tabs"
import { ScrollArea, ScrollBar } from "@lana/web/ui/scroll-area"
import {
  LayoutDashboard,
  History,
  Banknote,
  AlertTriangle,
  CalendarCheck,
  BookOpen,
  Activity,
} from "lucide-react"

import { CreditFacilityHeader, CreditFacilityDetailsContent } from "./details"
import { CreditFacilityCollateral } from "./collateral-card"
import { CreditFacilityTermsCard } from "./terms-card"
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
    currentCvl {
      __typename
      ... on FiniteCvlPct {
        value
      }
      ... on InfiniteCvlPct {
        isInfinite
      }
    }
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
      effectiveAnnualRate
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

const OVERVIEW = "overview"

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
    { id: "1", url: "/", tabLabel: t("tabs.history"), icon: <History className="h-4 w-4" /> },
    { id: "4", url: "/disbursals", tabLabel: t("tabs.disbursals"), icon: <Banknote className="h-4 w-4" /> },
    { id: "7", url: "/liquidations", tabLabel: t("tabs.liquidations"), icon: <AlertTriangle className="h-4 w-4" /> },
    { id: "5", url: "/repayment-plan", tabLabel: t("tabs.repaymentPlan"), icon: <CalendarCheck className="h-4 w-4" /> },
    { id: "6", url: "/ledger-accounts", tabLabel: t("tabs.ledgerAccounts"), icon: <BookOpen className="h-4 w-4" /> },
    { id: "7", url: "/events", tabLabel: t("tabs.events"), icon: <Activity className="h-4 w-4" /> },
  ]

  const { currentTab, handleTabChange: handleRoutedTabChange } = useTabNavigation(TABS, publicId)
  const [isOverview, setIsOverview] = useState(true)

  const activeTab = isOverview ? OVERVIEW : currentTab

  const handleTabChange = (value: string) => {
    if (value === OVERVIEW) {
      setIsOverview(true)
    } else {
      setIsOverview(false)
      handleRoutedTabChange(value)
    }
  }

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

  const facility = data.creditFacilityByPublicId

  return (
    <main className="max-w-7xl w-full mx-auto border-l border-r flex-1">
      <CreditFacilityHeader creditFacilityDetails={facility} />
      <Tabs value={activeTab} onValueChange={handleTabChange} className="gap-0">
        <ScrollArea>
          <TabsList className="bg-transparent rounded-none h-auto w-full justify-start p-0 border-b">
            {[
              { value: OVERVIEW, label: t("tabs.overview"), icon: <LayoutDashboard className="h-4 w-4" /> },
              ...TABS.map((tab) => ({ value: tab.url, label: tab.tabLabel, icon: tab.icon })),
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
          <CreditFacilityDetailsContent creditFacilityDetails={facility} />
          <div className="h-1 bg-secondary border-t" />
          <div className="flex flex-col md:flex-row w-full border-b">
            <div className="md:w-[50%] md:border-r">
              <FacilityCard creditFacility={facility} />
            </div>
            <div className="hidden md:block w-1 bg-secondary border-r" />
            <div className="md:flex-1">
              <CreditFacilityCollateral creditFacility={facility} />
            </div>
          </div>
          <div className="h-1 bg-secondary border-t" />
          <CreditFacilityTermsCard creditFacility={facility} />
        </TabsContent>
        {TABS.map((tab) => (
          <TabsContent key={tab.url} value={tab.url}>
            {children}
          </TabsContent>
        ))}
      </Tabs>
    </main>
  )
}
