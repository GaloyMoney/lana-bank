"use client"

import React, { useEffect, useState, useCallback, useMemo } from "react"
import { gql } from "@apollo/client"
import { PiWarningCircleFill, PiCheckCircleFill } from "react-icons/pi"

import { ReportCreateDialog } from "./create-dialog"

import { PageHeading } from "@/components/page-heading"
import { Select } from "@/components/primitive/select"
import {
  ReportProgress,
  useReportDownloadLinksMutation,
  useReportsQuery,
  Report,
} from "@/lib/graphql/generated"
import { Button } from "@/components/primitive/button"
import { formatDate } from "@/lib/utils"

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

const RegulatoryReportingPage: React.FC = () => {
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

  if (loading) return <p>Loading...</p>
  if (error) return <p className="text-destructive">{error.message}</p>

  return (
    <main>
      <div className="flex justify-between items-center mb-14">
        <PageHeading className="mb-0">Regulatory Reporting</PageHeading>
        <Button onClick={() => setOpenReportCreateDialog(true)}>Generate Report</Button>
      </div>
      {data?.reports && data.reports.length > 0 ? (
        <div className="max-w-[50rem] space-y-6">
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
        <p>No reports found</p>
      )}
      {generateLinkError && (
        <p className="text-destructive">Error: {generateLinkError.message}</p>
      )}
      <ReportCreateDialog
        setOpenReportCreateDialog={setOpenReportCreateDialog}
        openReportCreateDialog={openReportCreateDialog}
        refetch={refetchReports}
      />
    </main>
  )
}

export default RegulatoryReportingPage

const ReportSelector: React.FC<{
  reports: Array<{ reportId: string; createdAt: string }>
  selectedReport: string | undefined
  onSelectReport: (reportId: string) => void
}> = ({ reports, selectedReport, onSelectReport }) => {
  return (
    <KeyValueItem
      label="Select Report"
      value={
        <div className="w-80">
          <Select
            value={selectedReport}
            onChange={(event) => onSelectReport(event.target.value)}
          >
            {reports.length > 0 ? (
              reports.map((report) => (
                <option key={report.reportId} value={report.reportId}>
                  {formatDate(report.createdAt)}
                </option>
              ))
            ) : (
              <option value="">No reports found</option>
            )}
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
  return (
    <div className="space-y-6">
      <KeyValueItem label="Report ID" value={selectedReportDetails.reportId} />
      {selectedReportDetails.lastError && (
        <KeyValueItem label="Last Error" value={selectedReportDetails.lastError} />
      )}
      <KeyValueItem
        label="Status"
        value={formatStatus({
          reportProgress: selectedReportDetails.progress,
          createdAt: selectedReportDetails.createdAt,
        })}
      />
      <KeyValueItem
        label="Downloads"
        value={
          linksData && linksData.reportId === selectedReportDetails.reportId ? (
            <DownloadLinks links={linksData.links} />
          ) : (
            <Button
              onClick={onGenerateLinks}
              variant={
                selectedReportDetails.progress === ReportProgress.Running
                  ? "secondary"
                  : "primary"
              }
              disabled={
                generateLinkLoading ||
                selectedReportDetails.progress === ReportProgress.Running
              }
              loading={generateLinkLoading}
            >
              Generate Links
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
    <ul className="list-disc pl-5">
      {links.map((link, index) => (
        <li key={index}>
          <a
            href={link.url}
            download={link.reportName}
            target="_blank"
            rel="noopener noreferrer"
            className="underline"
          >
            {link.reportName}
          </a>
        </li>
      ))}
    </ul>
  )
}

const formatStatus = ({
  reportProgress,
  createdAt,
}: {
  reportProgress: ReportProgress
  createdAt: string
}) => {
  switch (reportProgress) {
    case ReportProgress.Running:
      return (
        <p className="text-warning flex items-center gap-2">
          <PiWarningCircleFill className="w-5 h-5" /> Running (last triggered on{" "}
          {formatDate(createdAt)})
        </p>
      )
    case ReportProgress.Complete:
      return (
        <p className="text-success flex items-center gap-2">
          <PiCheckCircleFill className="w-5 h-5" />
          Operational (last report created on {formatDate(createdAt)})
        </p>
      )
    default:
      return null
  }
}

const KeyValueItem: React.FC<{ label: string; value: React.ReactNode }> = ({
  label,
  value,
}) => {
  return (
    <div className="flex gap-4">
      <span className="w-32 font-semibold">{label}:</span>
      <div style={{ marginLeft: "calc(48px + 2rem)" }}>{value}</div>
    </div>
  )
}
