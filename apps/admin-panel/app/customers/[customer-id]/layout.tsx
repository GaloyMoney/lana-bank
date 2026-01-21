"use client"

import { gql } from "@apollo/client"
import { use, useEffect } from "react"
import { useTranslations } from "next-intl"

import { Tabs, TabsList, TabsTrigger, TabsContent } from "@lana/web/ui/tabs"
import { ScrollArea, ScrollBar } from "@lana/web/ui/scroll-area"

import { CustomerDetailsCard } from "./details"
import { KycStatus } from "./kyc-status"
import { DepositAccount } from "./deposit-account"

import { useTabNavigation } from "@/hooks/use-tab-navigation"
import {
  Customer as CustomerType,
  useCustomerKycUpdatedSubscription,
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
    email
    telegramId
    kycVerification
    activity
    level
    applicantId
    customerType
    createdAt
    publicId
    depositAccount {
      id
      status
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

  subscription CustomerKycUpdated($customerId: UUID!) {
    customerKycUpdated(customerId: $customerId) {
      customer {
        ...CustomerDetailsFragment
      }
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
    { id: "1", url: "/", tabLabel: t("tabs.creditFacilities") },
    {
      id: "2",
      url: "/pending-credit-facilities",
      tabLabel: t("tabs.pendingCreditFacilities"),
    },
    {
      id: "3",
      url: "/credit-facility-proposals",
      tabLabel: t("tabs.creditFacilityProposals"),
    },
    { id: "4", url: "/documents", tabLabel: t("tabs.documents") },
  ]

  const { "customer-id": customerId } = use(params)
  const { currentTab, handleTabChange } = useTabNavigation(TABS, customerId)

  const { setCustomLinks, resetToDefault } = useBreadcrumb()

  const { setCustomer } = useCreateContext()
  const { data, loading, error } = useGetCustomerBasicDetailsQuery({
    variables: { id: customerId },
  })

  useCustomerKycUpdatedSubscription(
    data?.customerByPublicId?.customerId
      ? { variables: { customerId: data.customerByPublicId.customerId } }
      : { skip: true },
  )

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
    <main className="max-w-7xl m-auto">
      <CustomerDetailsCard customer={data.customerByPublicId} />
      <div className="flex flex-col md:flex-row w-full gap-2 my-2">
        <KycStatus
          customerId={data.customerByPublicId.customerId}
          kycVerification={data.customerByPublicId.kycVerification}
          level={data.customerByPublicId.level}
          applicantId={data.customerByPublicId.applicantId}
        />
        {data.customerByPublicId.depositAccount ? (
          <DepositAccount
            balance={data.customerByPublicId.depositAccount.balance}
            publicId={data.customerByPublicId.depositAccount.publicId}
            status={data.customerByPublicId.depositAccount.status}
          />
        ) : (
          <span className="rounded-md bg-muted px-2 py-1 text-sm font-medium text-muted-foreground md:w-full w-1/4 text-center flex items-center justify-center">
            {tDepositAccount("noAccount")}
          </span>
        )}
      </div>
      <Tabs
        defaultValue={TABS[0].url}
        value={currentTab}
        onValueChange={handleTabChange}
        className="mt-2"
      >
        <ScrollArea>
          <div className="relative h-10">
            <TabsList className="flex absolute h-10">
              {TABS.map((tab) => (
                <TabsTrigger key={tab.id} value={tab.url} id={`tab-${tab.id}`}>
                  {tab.tabLabel}
                </TabsTrigger>
              ))}
            </TabsList>
          </div>
          <ScrollBar orientation="horizontal" />
        </ScrollArea>
        {TABS.map((tab) => (
          <TabsContent key={tab.id} value={tab.url}>
            {children}
          </TabsContent>
        ))}
      </Tabs>
    </main>
  )
}
