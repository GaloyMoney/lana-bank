"use client"

import { use } from "react"
import { FaChevronRight } from "react-icons/fa"
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
import { formatDate, fromISODateString, toISODateString } from "@lana/web/utils"

import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"

import {
  Report,
  useReportGenerateDownloadLinkMutation,
  useReportsByDateQuery,
} from "@/lib/graphql/generated"

gql`
  fragment ReportFields on Report {
    id
    reportId
    date
    pathInBucket
  }

  query ReportsByDate($date: Date!, $first: Int!, $after: String) {
    reportsByDate(date: $date, first: $first, after: $after) {
      edges {
        cursor
        node {
          ...ReportFields
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

  mutation reportGenerateDownloadLink($reportId: UUID!) {
    reportGenerateDownloadLink(reportId: $reportId)
  }
`

type ReportByDatePageProps = {
  params: Promise<{
    date: string
  }>
}

const ReportByDatePage = ({ params }: ReportByDatePageProps) => {
  const { date: _date } = use(params)
  const date = fromISODateString(_date)

  const t = useTranslations("ReportsByDate")

  const [generateDownloadLink] = useReportGenerateDownloadLinkMutation()

  const { data, loading, error, fetchMore } = useReportsByDateQuery({
    variables: {
      date: toISODateString(date),
      first: DEFAULT_PAGESIZE,
    },
  })

  return (
    <Card>
      <CardHeader className="flex flex-col md:flex-row md:justify-between md:items-center gap-4">
        <div className="flex flex-col gap-1">
          <CardTitle>
            {t("title", { date: formatDate(date, { includeTime: false }) })}
          </CardTitle>
          <CardDescription>{t("description")}</CardDescription>
        </div>
      </CardHeader>
      <CardContent>
        {error && <p className="text-destructive text-sm">{error?.message}</p>}
        <PaginatedTable<Report>
          columns={columns(t, generateDownloadLink)}
          data={data?.reportsByDate as PaginatedData<Report>}
          loading={loading}
          fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
          pageSize={DEFAULT_PAGESIZE}
        />
      </CardContent>
    </Card>
  )
}

export default ReportByDatePage

const columns = (
  t: ReturnType<typeof useTranslations>,
  generateDownloadLink: ReturnType<typeof useReportGenerateDownloadLinkMutation>[0],
): Column<Report>[] => [
  {
    key: "pathInBucket",
    label: t("reportName"),
    render: (pathInBucket) => {
      const pathInBucketParts = pathInBucket.split("/")
      const fileNameWithExt = pathInBucketParts[pathInBucketParts.length - 1]
      const norm = pathInBucketParts[pathInBucketParts.length - 2].replace(/_/g, " ")

      const ext = fileNameWithExt.split(".").pop()
      const fileName = fileNameWithExt.replace(`.${ext}`, "")
      const beautifiedFileName = fileName
        .replace(/_/g, " ")
        .split(" ")
        .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
        .join(" ")

      return (
        <div className="flex items-center gap-1">
          <strong>{norm.toUpperCase()}</strong>
          <FaChevronRight className="text-xs" />
          <span>{beautifiedFileName}</span>
        </div>
      )
    },
  },
  {
    key: "reportId",
    label: t("actions"),
    render: (_, { reportId, pathInBucket }) => {
      const ext = pathInBucket.split(".").pop()?.toUpperCase() ?? ""

      const getLink = async () => {
        const { data } = await generateDownloadLink({ variables: { reportId } })
        return data?.reportGenerateDownloadLink
      }

      return (
        <div className="flex items-center gap-2">
          {/* Download */}
          <Button
            variant="outline"
            onClick={async () => {
              const url = await getLink()
              if (!url) return toast.error(t("errorGeneratingLink"))
              const a = document.createElement("a")
              a.href = url
              a.download = ""
              a.click()
            }}
          >
            <HiDownload />
            <span className="uppercase">{ext}</span>
          </Button>

          {/* Preview / open */}
          <Button
            variant="outline"
            onClick={async () => {
              const url = await getLink()
              if (!url) return toast.error(t("errorGeneratingLink"))
              window.open(url, "_blank", "noopener")
            }}
          >
            <HiExternalLink />
            <span className="uppercase">{ext}</span>
          </Button>
        </div>
      )
    },
  },
]
