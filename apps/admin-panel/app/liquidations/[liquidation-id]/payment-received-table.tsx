"use client"

import React from "react"
import { useTranslations } from "next-intl"

import CardWrapper from "@/components/card-wrapper"
import Balance from "@/components/balance/balance"
import DataTable, { Column } from "@/components/data-table"
import { GetLiquidationDetailsQuery } from "@/lib/graphql/generated"

type PaymentReceived = NonNullable<
  GetLiquidationDetailsQuery["liquidation"]
>["receivedProceeds"][number]

type LiquidationProceedsReceivedTableProps = {
  paymentsReceived: PaymentReceived[]
}

export const LiquidationProceedsReceivedTable: React.FC<
  LiquidationProceedsReceivedTableProps
> = ({ paymentsReceived }) => {
  const t = useTranslations("Liquidations.LiquidationDetails.PaymentReceived")

  const columns: Column<PaymentReceived>[] = [
    {
      key: "amount",
      header: t("columns.amount"),
      width: "200px",
      render: (amount: PaymentReceived["amount"]) => (
        <Balance amount={amount} currency="usd" />
      ),
    },
    {
      key: "ledgerTxId",
      header: t("columns.ledgerTxId"),
      width: "250px",
      render: (txId: PaymentReceived["ledgerTxId"]) => (
        <span className="font-mono text-xs" title={txId}>
          {txId.slice(0, 8)}...{txId.slice(-8)}
        </span>
      ),
    },
  ]

  return (
    <div className="flex-1">
      <CardWrapper title={t("title")} description={t("description")}>
        <DataTable
          data={paymentsReceived}
          columns={columns}
          emptyMessage={t("messages.emptyTable")}
          navigateTo={(item) => `/ledger-transactions/${item.ledgerTxId}`}
        />
      </CardWrapper>
    </div>
  )
}
