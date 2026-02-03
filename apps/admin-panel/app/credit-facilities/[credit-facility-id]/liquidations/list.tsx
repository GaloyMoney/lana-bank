"use client"

import React from "react"
import { useTranslations } from "next-intl"

import DateWithTooltip from "@lana/web/components/date-with-tooltip"
import { Badge } from "@lana/web/ui/badge"

import Balance from "@/components/balance/balance"
import CardWrapper from "@/components/card-wrapper"
import DataTable, { Column } from "@/components/data-table"
import { GetCreditFacilityLiquidationsQuery } from "@/lib/graphql/generated"

type Liquidation = NonNullable<
  GetCreditFacilityLiquidationsQuery["creditFacilityByPublicId"]
>["liquidations"][number]

type CreditFacilityLiquidationsProps = {
  creditFacility: NonNullable<
    GetCreditFacilityLiquidationsQuery["creditFacilityByPublicId"]
  >
}

export const CreditFacilityLiquidations: React.FC<CreditFacilityLiquidationsProps> = ({
  creditFacility,
}) => {
  const t = useTranslations("CreditFacilities.CreditFacilityDetails.Liquidations")

  const columns: Column<Liquidation>[] = [
    {
      key: "completed",
      header: t("columns.status"),
      render: (completed: Liquidation["completed"]) => {
        return completed ? (
          <Badge variant="success">{t("status.completed")}</Badge>
        ) : (
          <Badge variant="warning">{t("status.inProgress")}</Badge>
        )
      },
    },
    {
      key: "expectedToReceive",
      header: t("columns.expectedToReceive"),
      render: (amount: Liquidation["expectedToReceive"]) => (
        <Balance amount={amount} currency="usd" />
      ),
    },
    {
      key: "sentTotal",
      header: t("columns.sentTotal"),
      render: (amount: Liquidation["sentTotal"]) => (
        <Balance amount={amount} currency="btc" />
      ),
    },
    {
      key: "amountReceived",
      header: t("columns.amountReceived"),
      render: (amount: Liquidation["amountReceived"]) => (
        <Balance amount={amount} currency="usd" />
      ),
    },
    {
      key: "createdAt",
      header: t("columns.createdAt"),
      render: (date: Liquidation["createdAt"]) => <DateWithTooltip value={date} />,
    },
  ]

  return (
    <CardWrapper title={t("title")} description={t("description")}>
      <DataTable
        data={creditFacility.liquidations}
        columns={columns}
        emptyMessage={t("messages.emptyTable")}
      />
    </CardWrapper>
  )
}
