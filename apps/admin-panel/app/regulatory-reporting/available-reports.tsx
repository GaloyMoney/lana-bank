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
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@lana/web/ui/dialog"
import { Label } from "@lana/web/ui/label"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@lana/web/ui/table"
import { formatSpacedSentenceCaseFromSnakeCase } from "@lana/web/utils"

import { AsOfDateSelector, getInitialAsOfDate } from "../balance-sheet/as-of-date-selector"

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

type AvailableReportDefinition = AvailableReportDefinitionsQuery["availableReportDefinitions"][number]
type ReportDefinitionGroup = {
  norm: string
  reports: AvailableReportDefinition[]
}

const formatReportName = (report: AvailableReportDefinition): string =>
  formatSpacedSentenceCaseFromSnakeCase(report.friendlyName)

const AvailableReports: React.FC = () => {
  const t = useTranslations("Reports")
  const initialAsOfDate = useMemo(() => getInitialAsOfDate(), [])
  const [selectedReport, setSelectedReport] = useState<AvailableReportDefinition | null>(null)
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
                  <div className="overflow-x-auto">
                    <Table>
                      <TableHeader>
                        <TableRow>
                          <TableHead>{t("availableReportsHeaders.report")}</TableHead>
                          <TableHead>{t("availableReportsHeaders.outputs")}</TableHead>
                          <TableHead>{t("availableReportsHeaders.asOfDate")}</TableHead>
                          <TableHead className="text-right">
                            {t("availableReportsHeaders.action")}
                          </TableHead>
                        </TableRow>
                      </TableHeader>
                      <TableBody>
                        {group.reports.map((report) => (
                          <TableRow key={report.reportDefinitionId}>
                            <TableCell>
                              <div className="flex flex-col gap-1">
                                <div className="font-medium">{formatReportName(report)}</div>
                                <div className="text-xs text-muted-foreground font-mono">
                                  {report.reportDefinitionId}
                                </div>
                              </div>
                            </TableCell>
                            <TableCell>
                              <div className="flex flex-wrap gap-2">
                                {report.outputs.map((output) => (
                                  <Badge
                                    key={`${report.reportDefinitionId}-${output.format}`}
                                    variant="secondary"
                                  >
                                    {String(output.format).toUpperCase()}
                                  </Badge>
                                ))}
                              </div>
                            </TableCell>
                            <TableCell>
                              {report.supportsAsOf
                                ? t("availableReportsValues.asOfDate.required")
                                : t("availableReportsValues.asOfDate.notUsed")}
                            </TableCell>
                            <TableCell className="text-right">
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
                            </TableCell>
                          </TableRow>
                        ))}
                      </TableBody>
                    </Table>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      <Dialog open={selectedReport !== null} onOpenChange={handleDialogOpenChange}>
        <DialogContent className="sm:max-w-[480px]">
          <DialogHeader>
            <DialogTitle>{t("ReportGeneration.selectAsOfDate")}</DialogTitle>
            <DialogDescription>
              {selectedReport
                ? t("ReportGeneration.selectAsOfDateDescription", {
                    report: formatReportName(selectedReport),
                  })
                : null}
            </DialogDescription>
          </DialogHeader>
          <div className="flex flex-col gap-3">
            <div className="flex flex-col gap-2">
              <Label>{t("ReportGeneration.asOfDate")}</Label>
              <AsOfDateSelector asOf={asOfDate} onDateChange={setAsOfDate} />
            </div>
          </div>
          <DialogFooter>
            <Button
              type="button"
              disabled={!selectedReport || generating}
              onClick={() => {
                if (!selectedReport) return
                handleGenerate(selectedReport.reportDefinitionId, asOfDate)
              }}
            >
              {t("ReportGeneration.generate")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  )
}

export { AvailableReports }
