"use client"

import React, { useEffect, useState, useCallback, useMemo } from "react"
import { gql } from "@apollo/client"
import { PiWarningCircleFill, PiCheckCircleFill } from "react-icons/pi"

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@lana/web/ui/select"

import { Button } from "@lana/web/ui/button"

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@lana/web/ui/card"
import { Skeleton } from "@lana/web/ui/skeleton"

import { useTranslations } from "next-intl"

import { formatDate } from "@lana/web/utils"

import { ReportCreateDialog } from "./create-dialog"

import {
  ReportProgress,
  useReportDownloadLinksMutation,
  useReportsQuery,
  Report,
} from "@/lib/graphql/generated"

gql`
  query Reports {
    reports {
      reportId
      createdAt
      lastError
      progress
    }
  }

  mutation ReportDownloadLinks($input: ReportDownloadLinksGenerateInput!) {
    reportDownloadLinksGenerate(input: $input) {
      reportId
      links {
        reportName
        url
      }
    }
  }
`

const REFETCH_INTERVAL = 5000

const LoadingSkeleton = () => {
  return (
    <div className="space-y-4 md:space-y-6">
      <div className="flex flex-col md:flex-row md:gap-4">
        <Skeleton className="h-6 w-32 mb-2 md:mb-0" />
        <div className="w-full md:ml-16">
          <Skeleton className="h-10 w-full md:w-80" />
        </div>
      </div>

      <div className="flex flex-col md:flex-row md:gap-4">
        <Skeleton className="h-6 w-32 mb-2 md:mb-0" />
        <div className="w-full md:ml-16">
          <Skeleton className="h-6 w-full md:w-64" />
        </div>
      </div>

      <div className="flex flex-col md:flex-row md:gap-4">
        <Skeleton className="h-6 w-32 mb-2 md:mb-0" />
        <div className="w-full md:ml-16">
          <div className="flex items-center gap-2">
            <Skeleton className="h-5 w-5 rounded-full" />
            <Skeleton className="h-6 w-full md:w-96" />
          </div>
        </div>
      </div>

      <div className="flex flex-col md:flex-row md:gap-4">
        <Skeleton className="h-6 w-32 mb-2 md:mb-0" />
        <div className="w-full md:ml-16">
          <Skeleton className="h-10 w-32" />
        </div>
      </div>
    </div>
  )
}
const RegulatoryReportingPage: React.FC = () => {
  const t = useTranslations("Reports")
  const { data, loading, error, refetch: refetchReports } = useReportsQuery()
  const [
    generateLinks,
    { data: linksData, error: generateLinkError, loading: generateLinkLoading },
  ] = useReportDownloadLinksMutation()

  const [selectedReport, setSelectedReport] = useState<string | undefined>(undefined)
  const [openReportCreateDialog, setOpenReportCreateDialog] = useState(false)

  const selectedReportDetails = useMemo(() => {
    return data?.reports?.find((r) => r.reportId === selectedReport)
  }, [data, selectedReport])

  useEffect(() => {
    if (data?.reports && data.reports.length > 0) {
      setSelectedReport(data.reports[0].reportId)
    }
  }, [data])

  useEffect(() => {
    let intervalId: NodeJS.Timeout | null = null

    if (selectedReportDetails?.progress === ReportProgress.Running) {
      intervalId = setInterval(() => {
        refetchReports()
      }, REFETCH_INTERVAL)
    }

    return () => {
      if (intervalId) clearInterval(intervalId)
    }
  }, [selectedReportDetails, refetchReports])

  const handleGenerateLinks = useCallback(async () => {
    if (!selectedReport) return

    try {
      await generateLinks({
        variables: { input: { reportId: selectedReport } },
      })
    } catch (error) {
      console.error("Error generating links:", error)
    }
  }, [selectedReport, generateLinks])

  if (error) return <p className="text-destructive px-4">{error.message}</p>

  return (
    <>
      <Card className="mx-4 md:mx-auto md:max-w-7xl">
        <CardHeader className="flex flex-col md:flex-row md:justify-between md:items-center gap-4">
          <div className="flex flex-col gap-1">
            <CardTitle>{t("title")}</CardTitle>
            <CardDescription>{t("description")}</CardDescription>
          </div>
          <Button
            data-testid="generate-report-button"
            variant="outline"
            className="w-full md:w-auto"
            onClick={() => setOpenReportCreateDialog(true)}
          >
            {t("generateReport")}
          </Button>
        </CardHeader>

        <CardContent>
          {loading && !data ? (
            <div className="w-full md:max-w-[50rem]">
              <LoadingSkeleton />
            </div>
          ) : data?.reports && data.reports.length > 0 ? (
            <div className="w-full md:max-w-[50rem] space-y-4 md:space-y-6">
              <ReportSelector
                reports={data?.reports || []}
                selectedReport={selectedReport}
                onSelectReport={setSelectedReport}
              />
              {selectedReport && selectedReportDetails && (
                <ReportDetails
                  selectedReportDetails={selectedReportDetails}
                  onGenerateLinks={handleGenerateLinks}
                  generateLinkLoading={generateLinkLoading}
                  linksData={linksData?.reportDownloadLinksGenerate}
                />
              )}
            </div>
          ) : (
            <p className="text-center md:text-left">{t("noReportsFound")}</p>
          )}
          {generateLinkError && (
            <p className="text-destructive mt-4">{generateLinkError.message}</p>
          )}
        </CardContent>
      </Card>

      <ReportCreateDialog
        setOpenReportCreateDialog={setOpenReportCreateDialog}
        openReportCreateDialog={openReportCreateDialog}
        refetch={refetchReports}
      />
    </>
  )
}

const ReportSelector: React.FC<{
  reports: Array<{ reportId: string; createdAt: string }>
  selectedReport: string | undefined
  onSelectReport: (reportId: string) => void
}> = ({ reports, selectedReport, onSelectReport }) => {
  const t = useTranslations("Reports")

  return (
    <KeyValueItem
      label={t("selectReport")}
      value={
        <div className="w-full md:w-80">
          <Select value={selectedReport} onValueChange={onSelectReport}>
            <SelectTrigger>
              <SelectValue placeholder={t("selectReportPlaceholder")} />
            </SelectTrigger>
            <SelectContent>
              {reports.length > 0 ? (
                reports.map((report) => (
                  <SelectItem key={report.reportId} value={report.reportId}>
                    {formatDate(report.createdAt)}
                  </SelectItem>
                ))
              ) : (
                <SelectItem value="">{t("noReportsFound")}</SelectItem>
              )}
            </SelectContent>
          </Select>
        </div>
      }
    />
  )
}

const ReportDetails: React.FC<{
  selectedReportDetails: Report
  onGenerateLinks: () => void
  generateLinkLoading: boolean
  linksData?: { reportId: string; links: Array<{ reportName: string; url: string }> }
}> = ({ selectedReportDetails, onGenerateLinks, generateLinkLoading, linksData }) => {
  const t = useTranslations("Reports")

  return (
    <div className="space-y-4 md:space-y-6">
      <KeyValueItem
        data-testid="report-id"
        label={t("reportId")}
        value={selectedReportDetails.reportId}
      />
      {selectedReportDetails.lastError && (
        <KeyValueItem
          data-testid="report-error"
          label={t("lastError")}
          value={selectedReportDetails.lastError}
        />
      )}
      <KeyValueItem
        data-testid="report-status"
        label={t("currentStatus")}
        value={
          <FormatStatus
            reportProgress={selectedReportDetails.progress}
            createdAt={selectedReportDetails.createdAt}
          />
        }
      />
      <KeyValueItem
        data-testid="report-downloads"
        label={t("downloads")}
        value={
          linksData && linksData.reportId === selectedReportDetails.reportId ? (
            <DownloadLinks links={linksData.links} />
          ) : (
            <Button
              onClick={onGenerateLinks}
              variant={
                selectedReportDetails.progress === ReportProgress.Running
                  ? "secondary"
                  : "default"
              }
              disabled={
                generateLinkLoading ||
                selectedReportDetails.progress === ReportProgress.Running
              }
              loading={generateLinkLoading}
              className="w-full md:w-auto"
            >
              {t("generateLinks")}
            </Button>
          )
        }
      />
    </div>
  )
}

const DownloadLinks: React.FC<{
  links: Array<{ reportName: string; url: string }>
}> = ({ links }) => {
  return (
    <ul className="list-disc pl-5 space-y-2">
      {links.map((link, index) => (
        <li key={index}>
          <a
            href={link.url}
            download={link.reportName}
            target="_blank"
            rel="noopener noreferrer"
            className="underline break-all"
          >
            {link.reportName}
          </a>
        </li>
      ))}
    </ul>
  )
}

const FormatStatus = ({
  reportProgress,
  createdAt,
}: {
  reportProgress: ReportProgress
  createdAt: string
}) => {
  const t = useTranslations("Reports.status")

  switch (reportProgress) {
    case ReportProgress.Running:
      return (
        <p className="text-warning flex items-center gap-2 flex-wrap">
          <PiWarningCircleFill className="w-5 h-5" />
          <span>{t("running", { date: formatDate(createdAt) })}</span>
        </p>
      )
    case ReportProgress.Complete:
      return (
        <p className="text-success flex items-center gap-2 flex-wrap">
          <PiCheckCircleFill className="w-5 h-5" />
          <span>{t("complete", { date: formatDate(createdAt) })}</span>
        </p>
      )
    default:
      return null
  }
}

const KeyValueItem: React.FC<{
  "label": string
  "value": React.ReactNode
  "data-testid"?: string
}> = ({ label, value, "data-testid": testId }) => {
  return (
    <div className="flex flex-col md:flex-row md:gap-4" data-testid={testId}>
      <span className="font-semibold mb-2 md:mb-0 md:w-32">{label}:</span>
      <div className="w-full md:ml-16">{value}</div>
    </div>
  )
}

export default RegulatoryReportingPage
