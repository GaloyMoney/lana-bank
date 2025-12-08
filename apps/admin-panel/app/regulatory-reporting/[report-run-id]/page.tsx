"use client"

import { use } from "react"
import { HiDownload, HiExternalLink } from "react-icons/hi"
import { gql } from "@apollo/client"
import { toast } from "sonner"

import { useTranslations } from "next-intl"
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "@lana/web/ui/card"
import { Button } from "@lana/web/ui/button"
import { formatDate, formatSpacedSentenceCaseFromSnakeCase } from "@lana/web/utils"

import DataTable, { Column } from "@/components/data-table"

import {
  Report,
  useReportFileGenerateDownloadLinkMutation,
  useReportRunByIdQuery,
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

const ReportRunPage = ({ params }: ReportRunPageProps) => {
  const { "report-run-id": reportRunId } = use(params)

  const t = useTranslations("ReportRun")

  const [generateDownloadLink] = useReportFileGenerateDownloadLinkMutation()

  const { data, loading, error } = useReportRunByIdQuery({
    variables: {
      reportRunId,
    },
  })

  if (loading && !data) return <TableLoadingSkeleton />

  const date = data?.reportRun?.startTime

  return (
    <Card>
      <CardHeader className="flex flex-col md:flex-row md:justify-between md:items-center gap-4">
        <div className="flex flex-col gap-1">
          <CardTitle>
            {t("title", {
              date: formatDate(date, { includeTime: true }),
            })}
          </CardTitle>
          {data?.reportRun?.state && (
            <CardDescription>
              {t(`state.${data?.reportRun?.state?.toLowerCase()}`)}
            </CardDescription>
          )}
        </div>
      </CardHeader>
      <CardContent>
        {error && <p className="text-destructive text-sm">{error?.message}</p>}
        <DataTable<Report>
          columns={columns(t, generateDownloadLink)}
          data={(data?.reportRun?.reports || []) as Report[]}
          emptyMessage={t("noReportsAvailable")}
        />
      </CardContent>
    </Card>
  )
}

export default ReportRunPage

const columns = (
  t: ReturnType<typeof useTranslations>,
  generateDownloadLink: ReturnType<typeof useReportFileGenerateDownloadLinkMutation>[0],
): Column<Report>[] => [
  {
    key: "norm",
    header: t("norm"),
    render: (norm) => (
      <strong>{formatSpacedSentenceCaseFromSnakeCase(norm).toUpperCase()}</strong>
    ),
  },
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
          {files.map((file) => (
            <>
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
            </>
          ))}
        </div>
      )
    },
  },
]
