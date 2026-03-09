"use client"

import { use, useEffect, useMemo } from "react"
import { HiDownload, HiExternalLink } from "react-icons/hi"
import { gql } from "@apollo/client"
import { toast } from "sonner"

import { useTranslations } from "next-intl"
import { Badge } from "@lana/web/ui/badge"
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "@lana/web/ui/card"
import { Button } from "@lana/web/ui/button"
import { formatDate, formatSpacedSentenceCaseFromSnakeCase } from "@lana/web/utils"
import { Separator } from "@lana/web/ui/separator"

import DataTable, { Column } from "@/components/data-table"

import {
  ReportRunByIdQuery,
  useReportFileGenerateDownloadLinkMutation,
  useReportRunByIdQuery,
  useReportRunUpdatedSubscription,
} from "@/lib/graphql/generated"
import { TableLoadingSkeleton } from "@/components/table-loading-skeleton"

gql`
  query ReportRunById($reportRunId: UUID!) {
    reportRun(id: $reportRunId) {
      id
      reportRunId
      state
      runType
      startTime
      requestedAsOfDate
      requestedReport {
        reportDefinitionId
        norm
        name
      }
      reports {
        id
        reportId
        externalId
        name
        norm
        files {
          extension
        }
      }
    }
  }

  mutation ReportFileGenerateDownloadLink($input: ReportFileGenerateDownloadLinkInput!) {
    reportFileGenerateDownloadLink(input: $input) {
      url
    }
  }
`

type ReportRunPageProps = {
  params: Promise<{
    "report-run-id": string
  }>
}

type ReportRow = NonNullable<ReportRunByIdQuery["reportRun"]>["reports"][number]

type ReportGroup = {
  norm: string
  reports: ReportRow[]
}

const ReportRunPage = ({ params }: ReportRunPageProps) => {
  const { "report-run-id": reportRunId } = use(params)

  const t = useTranslations("ReportRun")

  const [generateDownloadLink] = useReportFileGenerateDownloadLinkMutation()

  const { data, loading, error, refetch } = useReportRunByIdQuery({
    variables: {
      reportRunId,
    },
  })
  const { data: subscriptionData } = useReportRunUpdatedSubscription()
  const groupedReports = useMemo<ReportGroup[]>(() => {
    const reports = [...(data?.reportRun?.reports ?? [])].sort((left, right) => {
      return left.norm.localeCompare(right.norm) || left.name.localeCompare(right.name)
    })

    const reportsByNorm = new Map<string, ReportRow[]>()
    for (const report of reports) {
      const existingReports = reportsByNorm.get(report.norm)
      if (existingReports) {
        existingReports.push(report)
      } else {
        reportsByNorm.set(report.norm, [report])
      }
    }

    return Array.from(reportsByNorm, ([norm, reports]) => ({ norm, reports }))
  }, [data?.reportRun?.reports])

  useEffect(() => {
    if (subscriptionData?.reportRunUpdated?.reportRunId !== reportRunId) {
      return
    }
    refetch()
  }, [subscriptionData, refetch, reportRunId])

  if (loading && !data) return <TableLoadingSkeleton />

  const reportRun = data?.reportRun
  const date = reportRun?.startTime
  const requestedReport = reportRun?.requestedReport
  const requestedAsOfDate = reportRun?.requestedAsOfDate
  const title = requestedReport
    ? formatSpacedSentenceCaseFromSnakeCase(requestedReport.name)
    : t("title", {
        date: date ? formatDate(date, { includeTime: true }) : "",
      })
  const emptyMessage =
    reportRun?.state === "QUEUED" || reportRun?.state === "RUNNING"
      ? t("reportsPending")
      : t("noReportsAvailable")

  return (
    <Card>
      <CardHeader className="flex flex-col md:flex-row md:justify-between md:items-center gap-4">
        <div className="flex flex-col gap-1">
          <CardTitle>{title}</CardTitle>
          {reportRun?.state && (
            <CardDescription>
              {t(`state.${reportRun.state.toLowerCase()}`)}
            </CardDescription>
          )}
          <div className="flex flex-wrap items-center gap-2 text-sm text-muted-foreground">
            {requestedReport && <Badge variant="outline">{requestedReport.norm.toUpperCase()}</Badge>}
            {requestedAsOfDate && (
              <span>
                {t("asOfDate", {
                  date: formatDate(requestedAsOfDate, { includeTime: false }),
                })}
              </span>
            )}
            {date && (
              <span>
                {t("generatedAt", {
                  date: formatDate(date, { includeTime: true }),
                })}
              </span>
            )}
          </div>
        </div>
      </CardHeader>
      <CardContent>
        {error && <p className="text-destructive text-sm">{error?.message}</p>}
        {groupedReports.length === 0 ? (
          <DataTable<ReportRow>
            columns={columns(t, generateDownloadLink)}
            data={[]}
            emptyMessage={emptyMessage}
          />
        ) : (
          <div className="space-y-6">
            {groupedReports.map((group, index) => (
              <div key={group.norm} className="space-y-3">
                {index > 0 && <Separator />}
                <Badge variant="outline">{group.norm.toUpperCase()}</Badge>
                <DataTable<ReportRow>
                  columns={columns(t, generateDownloadLink)}
                  data={group.reports}
                />
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  )
}

export default ReportRunPage

const columns = (
  t: ReturnType<typeof useTranslations>,
  generateDownloadLink: ReturnType<typeof useReportFileGenerateDownloadLinkMutation>[0],
): Column<ReportRow>[] => [
  {
    key: "name",
    header: t("name"),
    render: (name) => formatSpacedSentenceCaseFromSnakeCase(name),
  },
  {
    key: "externalId",
    header: t("download"),
    render: (_, { reportId, files }) => {
      const getLink = (extension: string) => async () => {
        const { data } = await generateDownloadLink({
          variables: { input: { reportId, extension } },
        })
        return data?.reportFileGenerateDownloadLink.url
      }

      return (
        <div className="flex items-center gap-2">
          {[...files]
            .sort((left, right) => left.extension.localeCompare(right.extension))
            .map((file) => (
              <div key={`${reportId}-${file.extension}`} className="flex items-center gap-2">
                {/* Download */}
                <Button
                  variant="outline"
                  onClick={async () => {
                    const url = await getLink(file.extension)()
                    if (!url) return toast.error(t("errorGeneratingLink"))
                    const a = document.createElement("a")
                    a.href = url
                    a.download = ""
                    a.click()
                  }}
                >
                  <HiDownload />
                  <span className="uppercase">{file.extension}</span>
                </Button>

                {/* Preview / open */}
                <Button
                  variant="outline"
                  onClick={async () => {
                    const url = await getLink(file.extension)()
                    if (!url) return toast.error(t("errorGeneratingLink"))
                    window.open(url, "_blank", "noopener")
                  }}
                >
                  <HiExternalLink />
                  <span className="uppercase">{file.extension}</span>
                </Button>
              </div>
            ))}
        </div>
      )
    },
  },
]
