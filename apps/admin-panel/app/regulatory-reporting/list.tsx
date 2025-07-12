import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import { formatDate, toISODateString } from "@lana/web/utils"

import DataTable, { Column } from "@/components/data-table"
import { useReportListAvailableDatesQuery } from "@/lib/graphql/generated"
import { TableLoadingSkeleton } from "@/components/table-loading-skeleton"

gql`
  query reportListAvailableDates {
    reportListAvailableDates
  }
`

const AvailableDatesForReport: React.FC = () => {
  const t = useTranslations("Reports")

  const { data, loading, error } = useReportListAvailableDatesQuery()

  const dates = (data?.reportListAvailableDates || []).map((date) => new Date(date))

  const columns: Column<Date>[] = [
    {
      key: "getDate",
      header: t("availableReports"),
      render: (_, date) => formatDate(date, { includeTime: false }),
    },
  ]

  if (loading) return <TableLoadingSkeleton columns={2} />

  return (
    <div>
      {error && <p className="text-destructive text-sm">{error?.message}</p>}
      <DataTable<Date>
        data={dates}
        columns={columns}
        navigateTo={(date) => `/regulatory-reporting/${toISODateString(date)}`}
      />
    </div>
  )
}

export { AvailableDatesForReport }
