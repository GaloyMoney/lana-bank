"use client"

import { gql } from "@apollo/client"
import { use } from "react"
import { useTranslations } from "next-intl"

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@lana/web/ui/card"

import DataTable, { Column } from "@/components/data-table"
import { useCreditFacilityLedgerAccountsQuery } from "@/lib/graphql/generated"

gql`
  query CreditFacilityLedgerAccounts($publicId: PublicId!) {
    creditFacilityByPublicId(id: $publicId) {
      id
      ledgerAccounts {
        facilityAccountId
        disbursedReceivableNotYetDueAccountId
        disbursedReceivableDueAccountId
        disbursedReceivableOverdueAccountId
        disbursedDefaultedAccountId
        collateralAccountId
        collateralInLiquidationAccountId
        liquidatedCollateralAccountId
        proceedsFromLiquidationAccountId
        interestReceivableNotYetDueAccountId
        interestReceivableDueAccountId
        interestReceivableOverdueAccountId
        interestDefaultedAccountId
        interestIncomeAccountId
        feeIncomeAccountId
        paymentHoldingAccountId
        uncoveredOutstandingAccountId
      }
    }
  }
`

type LedgerAccountRow = {
  name: string
  ledgerAccountId: string
}

interface CreditFacilityLedgerAccountsPageProps {
  params: Promise<{
    "credit-facility-id": string
  }>
}

export default function CreditFacilityLedgerAccountsPage({
  params,
}: CreditFacilityLedgerAccountsPageProps) {
  const t = useTranslations("CreditFacilities.CreditFacilityDetails.LedgerAccounts")
  const { "credit-facility-id": publicId } = use(params)

  const { data, loading, error } = useCreditFacilityLedgerAccountsQuery({
    variables: { publicId },
  })

  if (error) return <div className="text-destructive">{error.message}</div>
  if (!data?.creditFacilityByPublicId?.ledgerAccounts) return null
  const { ledgerAccounts } = data.creditFacilityByPublicId

  const ledgerAccountsData: LedgerAccountRow[] = [
    { name: "Facility Account", ledgerAccountId: ledgerAccounts.facilityAccountId },
    {
      name: "Disbursed Receivable Not Yet Due",
      ledgerAccountId: ledgerAccounts.disbursedReceivableNotYetDueAccountId,
    },
    {
      name: "Disbursed Receivable Due",
      ledgerAccountId: ledgerAccounts.disbursedReceivableDueAccountId,
    },
    {
      name: "Disbursed Receivable Overdue",
      ledgerAccountId: ledgerAccounts.disbursedReceivableOverdueAccountId,
    },
    {
      name: "Disbursed Defaulted",
      ledgerAccountId: ledgerAccounts.disbursedDefaultedAccountId,
    },
    {
      name: "Collateral Account",
      ledgerAccountId: ledgerAccounts.collateralAccountId,
    },
    {
      name: "Collateral In Liquidation",
      ledgerAccountId: ledgerAccounts.collateralInLiquidationAccountId,
    },
    {
      name: "Liquidated Collateral",
      ledgerAccountId: ledgerAccounts.liquidatedCollateralAccountId,
    },
    {
      name: "Proceeds From Liquidation",
      ledgerAccountId: ledgerAccounts.proceedsFromLiquidationAccountId,
    },
    {
      name: "Interest Receivable Not Yet Due",
      ledgerAccountId: ledgerAccounts.interestReceivableNotYetDueAccountId,
    },
    {
      name: "Interest Receivable Due",
      ledgerAccountId: ledgerAccounts.interestReceivableDueAccountId,
    },
    {
      name: "Interest Receivable Overdue",
      ledgerAccountId: ledgerAccounts.interestReceivableOverdueAccountId,
    },
    {
      name: "Interest Defaulted",
      ledgerAccountId: ledgerAccounts.interestDefaultedAccountId,
    },
    {
      name: "Interest Income",
      ledgerAccountId: ledgerAccounts.interestIncomeAccountId,
    },
    { name: "Fee Income", ledgerAccountId: ledgerAccounts.feeIncomeAccountId },
    {
      name: "Payment Holding",
      ledgerAccountId: ledgerAccounts.paymentHoldingAccountId,
    },
    {
      name: "Uncovered Outstanding",
      ledgerAccountId: ledgerAccounts.uncoveredOutstandingAccountId,
    },
  ]

  const columns: Column<LedgerAccountRow>[] = [
    {
      key: "name",
      header: t("table.headers.name"),
      width: "65%",
    },
    {
      key: "ledgerAccountId",
      header: t("table.headers.id"),
    },
  ]

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("title")}</CardTitle>
        <CardDescription>{t("description")}</CardDescription>
      </CardHeader>
      <CardContent>
        <DataTable
          data={ledgerAccountsData}
          columns={columns}
          loading={loading}
          navigateTo={(account) => `/ledger-accounts/${account.ledgerAccountId}`}
        />
      </CardContent>
    </Card>
  )
}
