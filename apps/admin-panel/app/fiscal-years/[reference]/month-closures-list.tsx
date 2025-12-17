"use client"

import React from "react"
import { useTranslations } from "next-intl"

import DateWithTooltip from "@lana/web/components/date-with-tooltip"
import { formatUTCDateOnly } from "@lana/web/utils"

import { GetFiscalYearDetailsQuery } from "@/lib/graphql/generated"
import CardWrapper from "@/components/card-wrapper"
import DataTable, { Column } from "@/components/data-table"

type FiscalMonthClosure = NonNullable<
  GetFiscalYearDetailsQuery["fiscalYearByReference"]
>["monthClosures"][number]

type MonthClosuresListProps = {
  monthClosures: NonNullable<
    GetFiscalYearDetailsQuery["fiscalYearByReference"]
  >["monthClosures"]
}

const MonthClosuresList: React.FC<MonthClosuresListProps> = ({ monthClosures }) => {
  const t = useTranslations("FiscalYears.monthClosures")

  const columns: Column<FiscalMonthClosure>[] = [
    {
      key: "closedAsOf",
      header: t("table.headers.closedAsOf"),
      render: (value) => formatUTCDateOnly(value) ?? "-",
    },
    {
      key: "closedAt",
      header: t("table.headers.closedAt"),
      render: (value) => <DateWithTooltip value={value} />,
    },
  ]

  return (
    <CardWrapper title={t("title")}>
      <DataTable
        data={[...monthClosures].reverse()}
        columns={columns}
        emptyMessage={t("noClosures")}
      />
    </CardWrapper>
  )
}

export default MonthClosuresList
