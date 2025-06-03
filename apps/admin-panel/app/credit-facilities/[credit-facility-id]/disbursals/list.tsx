"use client"

import React from "react"
import { useTranslations } from "next-intl"

import DateWithTooltip from "@lana/web/components/date-with-tooltip"

import CardWrapper from "@/components/card-wrapper"
import { GetCreditFacilityDisbursalsQuery } from "@/lib/graphql/generated"
import Balance from "@/components/balance/balance"
import DataTable, { Column } from "@/components/data-table"
import { DisbursalStatusBadge } from "@/app/disbursals/status-badge"

type Disbursal = NonNullable<
  GetCreditFacilityDisbursalsQuery["creditFacility"]
>["disbursals"][number]

type CreditFacilityDisbursalsProps = {
  creditFacility: NonNullable<GetCreditFacilityDisbursalsQuery["creditFacility"]>
}

export const CreditFacilityDisbursals: React.FC<CreditFacilityDisbursalsProps> = ({
  creditFacility,
}) => {
  const t = useTranslations("CreditFacilities.CreditFacilityDetails.Disbursals")

  const columns: Column<Disbursal>[] = [
    {
      key: "amount",
      header: t("columns.amount"),
      render: (amount) => <Balance amount={amount} currency="usd" />,
    },
    {
      key: "createdAt",
      header: t("columns.createdAt"),
      render: (date) => <DateWithTooltip value={date} />,
    },
    {
      key: "status",
      header: t("columns.status"),
      align: "right",
      render: (_, disbursal) => {
        return <DisbursalStatusBadge status={disbursal.status} />
      },
    },
  ]

  return (
    <>
      <CardWrapper title={t("title")} description={t("description")}>
        <DataTable
          data={creditFacility.disbursals}
          columns={columns}
          emptyMessage={t("messages.emptyTable")}
          navigateTo={(disbursal) => `/disbursals/${disbursal.disbursalId}`}
        />
      </CardWrapper>
    </>
  )
}
