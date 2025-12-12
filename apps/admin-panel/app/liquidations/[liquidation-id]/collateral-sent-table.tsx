"use client"

import React from "react"
import { useTranslations } from "next-intl"

import CardWrapper from "@/components/card-wrapper"
import Balance from "@/components/balance/balance"
import DataTable, { Column } from "@/components/data-table"
import { GetLiquidationDetailsQuery } from "@/lib/graphql/generated"

type CollateralSent = NonNullable<
  GetLiquidationDetailsQuery["liquidation"]
>["sentCollateral"][number]

type LiquidationCollateralSentTableProps = {
  collateralSent: CollateralSent[]
}

export const LiquidationCollateralSentTable: React.FC<
  LiquidationCollateralSentTableProps
> = ({ collateralSent }) => {
  const t = useTranslations("Liquidations.LiquidationDetails.CollateralSent")

  const columns: Column<CollateralSent>[] = [
    {
      key: "amount",
      header: t("columns.amount"),
      width: "200px",
      render: (amount: CollateralSent["amount"]) => (
        <Balance amount={amount} currency="btc" />
      ),
    },
    {
      key: "ledgerTxId",
      header: t("columns.ledgerTxId"),
      width: "250px",
      render: (txId: CollateralSent["ledgerTxId"]) => (
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
          data={collateralSent}
          columns={columns}
          emptyMessage={t("messages.emptyTable")}
          navigateTo={(item) => `/ledger-transactions/${item.ledgerTxId}`}
        />
      </CardWrapper>
    </div>
  )
}
