"use client"

import { useMemo, useState } from "react"

import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"
import { toast } from "sonner"

import { Badge } from "@lana/web/ui/badge"
import { Button } from "@lana/web/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@lana/web/ui/card"
import { Separator } from "@lana/web/ui/separator"
import { formatSpacedSentenceCaseFromSnakeCase } from "@lana/web/utils"

import { GenerateReportDialog } from "./generate-report-dialog"

import DataTable, { Column } from "@/components/data-table"
import { getInitialAsOfDate } from "@/components/as-of-date-selector"

import {
  ReportRunsDocument,
  AvailableReportDefinitionsQuery,
  useAvailableReportDefinitionsQuery,
  useReportGenerateMutation,
} from "@/lib/graphql/generated"

gql`
  query AvailableReportDefinitions {
    availableReportDefinitions {
      reportDefinitionId
      norm
      id
      friendlyName
      supportsAsOf
      outputs {
        format
      }
    }
  }

  mutation ReportGenerate($input: TriggerReportRunInput!) {
    triggerReportRun(input: $input) {
      runId
    }
  }
`

type AvailableReportDefinition =
  AvailableReportDefinitionsQuery["availableReportDefinitions"][number]
type ReportDefinitionGroup = {
  norm: string
  reports: AvailableReportDefinition[]
}

const formatReportName = (report: AvailableReportDefinition): string =>
  formatSpacedSentenceCaseFromSnakeCase(report.friendlyName)

const AvailableReports: React.FC = () => {
  const t = useTranslations("Reports")
  const initialAsOfDate = useMemo(() => getInitialAsOfDate(), [])
  const [selectedReport, setSelectedReport] =
    useState<AvailableReportDefinition | null>(null)
  const [asOfDate, setAsOfDate] = useState(initialAsOfDate)

  const { data, loading, error } = useAvailableReportDefinitionsQuery()
  const [generateReport, { loading: generating }] = useReportGenerateMutation({
    refetchQueries: [ReportRunsDocument],
  })

  const handleGenerate = (reportDefinitionId: string, nextAsOfDate?: string): void => {
    generateReport({
      variables: {
        input: {
          reportDefinitionId,
          asOfDate: nextAsOfDate,
        },
      },
    })
      .then(() => {
        toast.success(t("ReportGeneration.reportGenerationHasBeenTriggered"))
        setSelectedReport(null)
      })
      .catch(() => {
        toast.error(t("ReportGeneration.reportGenerationFailed"))
      })
  }

  const handleDialogOpenChange = (open: boolean) => {
    if (open) return
    setSelectedReport(null)
    setAsOfDate(initialAsOfDate)
  }

  const reportDefinitions = data?.availableReportDefinitions
  const groupedReportDefinitions = useMemo<ReportDefinitionGroup[]>(() => {
    const availableReportDefinitions = reportDefinitions ?? []
    const sortedReports = [...availableReportDefinitions].sort((left, right) => {
      return (
        left.norm.localeCompare(right.norm) ||
        formatReportName(left).localeCompare(formatReportName(right))
      )
    })

    const reportsByNorm = new Map<string, AvailableReportDefinition[]>()
    for (const report of sortedReports) {
      const existingReports = reportsByNorm.get(report.norm)
      if (existingReports) {
        existingReports.push(report)
      } else {
        reportsByNorm.set(report.norm, [report])
      }
    }

    return Array.from(reportsByNorm, ([norm, reports]) => ({ norm, reports }))
  }, [reportDefinitions])

  const columns: Column<AvailableReportDefinition>[] = [
    {
      key: "friendlyName",
      header: t("availableReportsHeaders.report"),
      render: (_, report) => (
        <div className="flex flex-col gap-1">
          <div className="font-medium">{formatReportName(report)}</div>
          <div className="text-xs text-muted-foreground font-mono">
            {report.reportDefinitionId}
          </div>
        </div>
      ),
    },
    {
      key: "outputs",
      header: t("availableReportsHeaders.outputs"),
      render: (outputs, report) => (
        <div className="flex flex-wrap gap-2">
          {outputs.map((output) => (
            <Badge
              key={`${report.reportDefinitionId}-${output.format}`}
              variant="secondary"
            >
              {String(output.format).toUpperCase()}
            </Badge>
          ))}
        </div>
      ),
    },
    {
      key: "supportsAsOf",
      header: t("availableReportsHeaders.asOfDate"),
      render: (supportsAsOf) =>
        supportsAsOf
          ? t("availableReportsValues.asOfDate.required")
          : t("availableReportsValues.asOfDate.notUsed"),
    },
    {
      key: "reportDefinitionId",
      header: t("availableReportsHeaders.action"),
      align: "right",
      render: (_, report) => (
        <Button
          type="button"
          disabled={generating}
          onClick={() => {
            if (report.supportsAsOf) {
              setSelectedReport(report)
              return
            }
            handleGenerate(report.reportDefinitionId)
          }}
        >
          {report.supportsAsOf
            ? t("ReportGeneration.generateAsOf")
            : t("ReportGeneration.generate")}
        </Button>
      ),
    },
  ]

  return (
    <>
      <Card>
        <CardHeader>
          <CardTitle>{t("availableReports")}</CardTitle>
          <CardDescription>{t("availableReportsDescription")}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {error && <p className="text-destructive text-sm">{error.message}</p>}
          {!loading && (reportDefinitions?.length ?? 0) === 0 ? (
            <p className="text-sm text-muted-foreground">{t("noReportsAvailable")}</p>
          ) : (
            <div className="space-y-6">
              {groupedReportDefinitions.map((group, index) => (
                <div key={group.norm} className="space-y-3">
                  {index > 0 && <Separator />}
                  <Badge variant="outline">{group.norm.toUpperCase()}</Badge>
                  <DataTable<AvailableReportDefinition>
                    data={group.reports}
                    columns={columns}
                    loading={loading}
                  />
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      <GenerateReportDialog
        open={selectedReport !== null}
        onOpenChange={handleDialogOpenChange}
        reportName={selectedReport ? formatReportName(selectedReport) : null}
        asOfDate={asOfDate}
        onAsOfDateChange={setAsOfDate}
        onGenerate={() => {
          if (!selectedReport) return
          handleGenerate(selectedReport.reportDefinitionId, asOfDate)
        }}
        generating={generating}
      />
    </>
  )
}

export { AvailableReports, formatReportName }
