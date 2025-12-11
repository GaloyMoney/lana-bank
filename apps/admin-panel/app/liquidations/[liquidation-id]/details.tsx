"use client"

import React from "react"
import { useTranslations } from "next-intl"

import { Badge } from "@lana/web/ui/badge"
import DateWithTooltip from "@lana/web/components/date-with-tooltip"

import { DetailsCard, DetailItemProps } from "@/components/details"
import Balance from "@/components/balance/balance"

import { GetLiquidationDetailsQuery } from "@/lib/graphql/generated"

type LiquidationDetailsProps = {
  liquidation: NonNullable<GetLiquidationDetailsQuery["liquidation"]>
}

export const LiquidationDetailsCard: React.FC<LiquidationDetailsProps> = ({
  liquidation,
}) => {
  const t = useTranslations("Liquidations.LiquidationDetails.DetailsCard")

  const details: DetailItemProps[] = [
    {
      label: t("details.status"),
      value: liquidation.completed ? (
        <Badge variant="success">{t("status.completed")}</Badge>
      ) : (
        <Badge variant="warning">{t("status.inProgress")}</Badge>
      ),
    },
    {
      label: t("details.expectedToReceive"),
      value: <Balance amount={liquidation.expectedToReceive} currency="usd" />,
    },
    {
      label: t("details.sentTotal"),
      value: <Balance amount={liquidation.sentTotal} currency="btc" />,
    },
    {
      label: t("details.receivedTotal"),
      value: <Balance amount={liquidation.receivedTotal} currency="usd" />,
    },
    {
      label: t("details.createdAt"),
      value: <DateWithTooltip value={liquidation.createdAt} />,
    },
  ]

  return <DetailsCard title={t("title")} details={details} />
}
