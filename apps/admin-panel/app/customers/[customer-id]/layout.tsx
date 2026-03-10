"use client"

import { gql } from "@apollo/client"
import { use, useEffect, useState } from "react"
import { useTranslations } from "next-intl"

import { Tabs, TabsList, TabsTrigger, TabsContent } from "@lana/web/ui/tabs"
import { ScrollArea, ScrollBar } from "@lana/web/ui/scroll-area"
import {
  LayoutDashboard,
  CreditCard,
  Clock,
  FileText,
  Files,
  Activity,
} from "lucide-react"

import { CustomerHeader, CustomerDetailsContent } from "./details"
import { CustomerPersonalInfoCard } from "./personal-info-card"
import { CustomerCompanyInfoCard } from "./company-info-card"
import { KycStatus } from "./kyc-status"
import { DepositAccount } from "./deposit-account"

import { useTabNavigation } from "@/hooks/use-tab-navigation"
import {
  Customer as CustomerType,
  CustomerType as CustomerTypeEnum,
  useGetCustomerBasicDetailsQuery,
} from "@/lib/graphql/generated"
import { useCreateContext } from "@/app/create"
import { useBreadcrumb } from "@/app/breadcrumb-provider"
import { DetailsPageSkeleton } from "@/components/details-page-skeleton"
import { PublicIdBadge } from "@/components/public-id-badge"

gql`
  fragment CustomerDetailsFragment on Customer {
    id
    customerId
    status
    email
    telegramHandle
    level
    applicantId
    customerType
    createdAt
    publicId
    personalInfo {
      firstName
      lastName
      dateOfBirth
      nationality
      address
      companyName
    }
    depositAccount {
      id
      status
      activity
      publicId
      depositAccountId
      balance {
        settled
        pending
      }
      ledgerAccounts {
        depositAccountId
        frozenDepositAccountId
      }
    }
  }

  query GetCustomerBasicDetails($id: PublicId!) {
    customerByPublicId(id: $id) {
      ...CustomerDetailsFragment
    }
  }
`

export default function CustomerLayout({
  children,
  params,
}: {
  children: React.ReactNode
  params: Promise<{ "customer-id": string }>
}) {
  const t = useTranslations("Customers.CustomerDetails.layout")
  const navTranslations = useTranslations("Sidebar.navItems")
  const tDepositAccount = useTranslations("Customers.CustomerDetails.depositAccount")

  const TABS = [
    { id: "1", url: "/", tabLabel: t("tabs.creditFacilities"), icon: <CreditCard className="h-4 w-4" /> },
    {
      id: "2",
      url: "/pending-credit-facilities",
      tabLabel: t("tabs.pendingCreditFacilities"),
      icon: <Clock className="h-4 w-4" />,
    },
    {
      id: "3",
      url: "/credit-facility-proposals",
      tabLabel: t("tabs.creditFacilityProposals"),
      icon: <FileText className="h-4 w-4" />,
    },
    { id: "4", url: "/documents", tabLabel: t("tabs.documents"), icon: <Files className="h-4 w-4" /> },
    { id: "5", url: "/events", tabLabel: t("tabs.events"), icon: <Activity className="h-4 w-4" /> },
  ]

  const OVERVIEW = "overview"

  const { "customer-id": customerId } = use(params)
  const { currentTab, handleTabChange: handleRoutedTabChange } = useTabNavigation(TABS, customerId)
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

  const { setCustomLinks, resetToDefault } = useBreadcrumb()

  const { setCustomer } = useCreateContext()
  const { data, loading, error } = useGetCustomerBasicDetailsQuery({
    variables: { id: customerId },
  })

  useEffect(() => {
    if (data?.customerByPublicId) setCustomer(data.customerByPublicId as CustomerType)
    return () => setCustomer(null)
  }, [data?.customerByPublicId, setCustomer])

  useEffect(() => {
    if (data?.customerByPublicId) {
      const currentTabData = TABS.find((tab) => tab.url === currentTab)
      setCustomLinks([
        { title: navTranslations("customers"), href: "/customers" },
        {
          title: <PublicIdBadge publicId={data.customerByPublicId.publicId} />,
          href: `/customers/${customerId}`,
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
  }, [data?.customerByPublicId, currentTab])

  if (loading && !data) return <DetailsPageSkeleton detailItems={3} tabs={6} />
  if (error) return <div className="text-destructive">{t("errors.error")}</div>
  if (!data || !data.customerByPublicId) return null

  return (
    <main className="max-w-7xl w-full mx-auto border-l border-r flex-1">
      <CustomerHeader customer={data.customerByPublicId} />
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
          <CustomerDetailsContent customer={data.customerByPublicId} />
          <div className="h-1 bg-secondary border-b" />
          <div className="flex flex-col md:flex-row w-full border-b">
            <div className="md:w-[33%] md:border-r">
              <KycStatus
                level={data.customerByPublicId.level}
                applicantId={data.customerByPublicId.applicantId}
              />
            </div>
            <div className="hidden md:block w-1 bg-secondary border-r" />
            <div className="md:flex-1">
              {data.customerByPublicId.customerType === CustomerTypeEnum.Individual ? (
                <CustomerPersonalInfoCard customer={data.customerByPublicId} />
              ) : (
                <CustomerCompanyInfoCard customer={data.customerByPublicId} />
              )}
            </div>
          </div>
          <div className="h-1 bg-secondary border-b" />
          {data.customerByPublicId.depositAccount ? (
            <DepositAccount
              balance={data.customerByPublicId.depositAccount.balance}
              publicId={data.customerByPublicId.depositAccount.publicId}
              status={data.customerByPublicId.depositAccount.status}
              activity={data.customerByPublicId.depositAccount.activity}
            />
          ) : (
            <div className="p-4 border-b text-sm font-medium text-muted-foreground text-center">
              {tDepositAccount("noAccount")}
            </div>
          )}
        </TabsContent>
        {TABS.map((tab) => (
          <TabsContent key={tab.id} value={tab.url}>
            {children}
          </TabsContent>
        ))}
      </Tabs>
    </main>
  )
}
