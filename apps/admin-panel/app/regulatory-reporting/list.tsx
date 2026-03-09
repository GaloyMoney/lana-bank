import { useEffect, useState } from "react"

import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@lana/web/ui/card"
import { formatDate, formatSpacedSentenceCaseFromSnakeCase } from "@lana/web/utils"

import {
  ReportRun,
  ReportRunsSort,
  SortDirection,
  useReportRunsQuery,
  useReportRunUpdatedSubscription,
} from "@/lib/graphql/generated"

import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"
import { camelToScreamingSnake } from "@/lib/utils"

gql`
  query ReportRuns($first: Int!, $after: String, $sort: ReportRunsSort) {
    reportRuns(first: $first, after: $after, sort: $sort) {
      edges {
        cursor
        node {
          id
          reportRunId
          startTime
          requestedAsOfDate
          requestedReport {
            reportDefinitionId
            norm
            name
          }
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

  subscription ReportRunUpdated {
    reportRunUpdated {
      reportRunId
    }
  }
`

const AvailableReportRuns: React.FC = () => {
  const t = useTranslations("Reports")
  const [sortBy, setSortBy] = useState<ReportRunsSort | null>(null)

  const { data, loading, error, fetchMore, refetch } = useReportRunsQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
      sort: sortBy,
    },
  })

  const { data: subscriptionData } = useReportRunUpdatedSubscription()

  useEffect(() => {
    if (!subscriptionData?.reportRunUpdated) {
      return
    }
    refetch()
  }, [subscriptionData, refetch])

  const columns: Column<ReportRun>[] = [
    {
      key: "startTime",
      label: t("listHeaders.generatedAt"),
      sortable: true,
      render: (startTime) => {
        return startTime ? formatDate(startTime) : t("starting")
      },
    },
    {
      key: "requestedReport",
      label: t("listHeaders.report"),
      render: (requestedReport) =>
        requestedReport
          ? `${requestedReport.norm.toUpperCase()} / ${formatSpacedSentenceCaseFromSnakeCase(requestedReport.name)}`
          : t("listValues.report.allReports"),
    },
    {
      key: "requestedAsOfDate",
      label: t("listHeaders.asOfDate"),
      render: (requestedAsOfDate) =>
        requestedAsOfDate
          ? formatDate(requestedAsOfDate, { includeTime: false })
          : t("listValues.asOfDate.notApplicable"),
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
    <Card>
      <CardHeader>
        <CardTitle>{t("recentRuns")}</CardTitle>
        <CardDescription>{t("recentRunsDescription")}</CardDescription>
      </CardHeader>
      <CardContent>
        {error && <p className="text-destructive text-sm">{error?.message}</p>}
        <PaginatedTable<ReportRun>
          columns={columns}
          data={data?.reportRuns as PaginatedData<ReportRun>}
          loading={!data && loading}
          pageSize={DEFAULT_PAGESIZE}
          fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
          navigateTo={(reportRun) => `/regulatory-reporting/${reportRun.reportRunId}`}
          onSort={(column, direction) => {
            setSortBy({
              by: camelToScreamingSnake(column as string) as ReportRunsSort["by"],
              direction: direction as SortDirection,
            })
          }}
        />
      </CardContent>
    </Card>
  )
}

export { AvailableReportRuns }
