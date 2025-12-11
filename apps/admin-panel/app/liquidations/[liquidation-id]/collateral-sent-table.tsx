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
      render: (amount: CollateralSent["amount"]) => (
        <Balance amount={amount} currency="btc" />
      ),
    },
    {
      key: "ledgerTxId",
      header: t("columns.ledgerTxId"),
      render: (txId: CollateralSent["ledgerTxId"]) => (
        <span className="font-mono text-xs">{txId}</span>
      ),
    },
  ]

  return (
    <CardWrapper title={t("title")} description={t("description")}>
      <DataTable
        data={collateralSent}
        columns={columns}
        emptyMessage={t("messages.emptyTable")}
      />
    </CardWrapper>
  )
}
