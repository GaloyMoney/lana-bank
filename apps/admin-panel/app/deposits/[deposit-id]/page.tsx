"use client"

import { gql } from "@apollo/client"
import { use, useEffect } from "react"
import { useTranslations } from "next-intl"

import { Tabs, TabsList, TabsTrigger, TabsContent } from "@lana/web/ui/tabs"

import LedgerTransactions from "../../../components/ledger-transactions"

import DepositDetailsCard from "./details"
import { DepositEventHistory } from "./event-history"

import { NotFound } from "@/components/not-found"

import { useGetDepositDetailsQuery } from "@/lib/graphql/generated"
import { DetailsPageSkeleton } from "@/components/details-page-skeleton"
import { useBreadcrumb } from "@/app/breadcrumb-provider"
import { PublicIdBadge } from "@/components/public-id-badge"

gql`
  fragment DepositDetailsPageFragment on Deposit {
    id
    depositId
    publicId
    amount
    createdAt
    reference
    status
    ledgerTransactions {
      ...LedgerTransactionFields
    }
    account {
      id
      publicId
      customer {
        id
        customerId
        publicId
        applicantId
        email
        depositAccount {
          balance {
            settled
            pending
          }
        }
      }
    }
  }

  query GetDepositDetails($publicId: PublicId!) {
    depositByPublicId(id: $publicId) {
      ...DepositDetailsPageFragment
    }
  }
`

function DepositPage({
  params,
}: {
  params: Promise<{
    "deposit-id": string
  }>
}) {
  const { "deposit-id": publicId } = use(params)
  const { setCustomLinks, resetToDefault } = useBreadcrumb()
  const navTranslations = useTranslations("Sidebar.navItems")
  const tLedger = useTranslations("LedgerTransactions")
  const tEventHistory = useTranslations("Deposits.DepositDetails.eventHistory")

  const { data, loading, error } = useGetDepositDetailsQuery({
    variables: { publicId },
  })

  useEffect(() => {
    if (data?.depositByPublicId) {
      setCustomLinks([
        { title: navTranslations("deposits"), href: "/deposits" },
        {
          title: <PublicIdBadge publicId={data.depositByPublicId.publicId} />,
          isCurrentPage: true,
        },
      ])
    }
    return () => {
      resetToDefault()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [data?.depositByPublicId])

  if (loading && !data) {
    return <DetailsPageSkeleton tabs={0} tabsCards={0} />
  }
  if (error) return <div className="text-destructive">{error.message}</div>
  if (!data?.depositByPublicId) return <NotFound />

  return (
    <main className="max-w-7xl m-auto space-y-2">
      <DepositDetailsCard deposit={data.depositByPublicId} />
      <Tabs defaultValue="ledger">
        <TabsList>
          <TabsTrigger value="ledger">{tLedger("title")}</TabsTrigger>
          <TabsTrigger value="events">{tEventHistory("title")}</TabsTrigger>
        </TabsList>
        <TabsContent value="ledger">
          <LedgerTransactions
            ledgerTransactions={data.depositByPublicId.ledgerTransactions}
          />
        </TabsContent>
        <TabsContent value="events">
          <DepositEventHistory depositPublicId={publicId} />
        </TabsContent>
      </Tabs>
    </main>
  )
}

export default DepositPage
