import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import { formatDate } from "@lana/web/utils"

import { ReportRun, useReportRunsQuery } from "@/lib/graphql/generated"

import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"

gql`
  query ReportRuns($first: Int!, $after: String) {
    reportRuns(first: $first, after: $after) {
      edges {
        cursor
        node {
          id
          reportRunId
          startTime
          runType
          state
        }
      }
      pageInfo {
        endCursor
        startCursor
        hasNextPage
        hasPreviousPage
      }
    }
  }
`

const AvailableReportRuns: React.FC = () => {
  const t = useTranslations("Reports")

  const { data, loading, error, fetchMore } = useReportRunsQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
    },
    pollInterval: 5000,
  })

  const columns: Column<ReportRun>[] = [
    {
      key: "startTime",
      label: t("listHeaders.generatedAt"),
      render: (startTime) => {
        return startTime ? formatDate(startTime) : t("starting")
      },
    },
    {
      key: "runType",
      label: t("listHeaders.runType"),
      render: (runType) => runType && t(`listValues.runType.${runType?.toLowerCase()}`),
    },
    {
      key: "state",
      label: t("listHeaders.state"),
      render: (state) => state && t(`listValues.state.${state?.toLowerCase()}`),
    },
  ]

  return (
    <div>
      {error && <p className="text-destructive text-sm">{error?.message}</p>}
      <PaginatedTable<ReportRun>
        columns={columns}
        data={data?.reportRuns as PaginatedData<ReportRun>}
        loading={!data && loading}
        pageSize={DEFAULT_PAGESIZE}
        fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
        navigateTo={(reportRun) => `/regulatory-reporting/${reportRun.reportRunId}`}
      />
    </div>
  )
}

export { AvailableReportRuns }
