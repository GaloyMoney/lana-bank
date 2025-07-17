"use client"

import { useState } from "react"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"
import { toast } from "sonner"
import { FiAlertTriangle } from "react-icons/fi"

import { Button } from "@lana/web/ui/button"
import { Alert, AlertDescription } from "@lana/web/ui/alert"
import { Badge } from "@lana/web/ui/badge"
import { formatDate } from "@lana/web/utils"

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@lana/web/ui/dialog"

import {
  useReportGenerateMutation,
  useReportGenerationJobStatusQuery,
} from "@/lib/graphql/generated"

gql`
  mutation ReportGenerate {
    reportGenerate {
      runId
    }
  }

  query ReportGenerationJobStatus {
    reportGenerationJobStatus {
      running
      runType
      runStartedAt
      logs
      error
      lastRun {
        runType
        runStartedAt
        status
        logs
      }
    }
  }
`

const POLL_INTERVAL = 5000

const ReportGeneration: React.FC = () => {
  const [triggered, setTriggered] = useState(false)
  const [openStatusModal, setOpenStatusModal] = useState(false)

  const t = useTranslations("Reports.ReportGeneration")

  const [generateReport, { loading: generateLoading }] = useReportGenerateMutation()
  const {
    data,
    loading,
    refetch: refetchStatus,
  } = useReportGenerationJobStatusQuery({
    pollInterval: POLL_INTERVAL,
  })

  const triggerGenerateReport = async () => {
    try {
      setTriggered(true)
      await generateReport()
      toast.success(t("reportGenerationHasBeenTriggered"))
      await refetchStatus()
      setTriggered(false)
    } catch (e) {
      toast.error(
        t("reportGenerationFailed", { e: e instanceof Error ? e.message : String(e) }),
      )
    }
  }

  if (loading || !data) return <Button disabled>{t("generate")}</Button>

  const currentRunRunning =
    data.reportGenerationJobStatus.running || generateLoading || triggered
  const lastRunFailed =
    data.reportGenerationJobStatus.lastRun?.status.toLowerCase() === "failed"

  return (
    <>
      <StatusModal
        openStatusModal={openStatusModal}
        setOpenStatusModal={setOpenStatusModal}
      />
      <div className="flex items-center gap-1 rounded-md">
        {currentRunRunning ? (
          <Badge
            variant="secondary"
            className="p-2 cursor-pointer hover:bg-gray-300 transition duration-300 ease-in-out"
            onClick={() => setOpenStatusModal(true)}
          >
            <span className="inset-0 flex items-center justify-center">
              <span className="inline-block w-4 h-4 border-2 border-t-transparent border-current rounded-full animate-spin" />
            </span>
          </Badge>
        ) : (
          lastRunFailed && (
            <Alert
              variant="destructive"
              className="p-2 cursor-pointer hover:bg-red-300 transition duration-300 ease-in-out"
              onClick={() => setOpenStatusModal(true)}
            >
              <AlertDescription>
                <FiAlertTriangle className="h-4 w-4" />
              </AlertDescription>
            </Alert>
          )
        )}
        <Button
          type="button"
          disabled={currentRunRunning || triggered}
          onClick={triggerGenerateReport}
        >
          {t("generate")}
        </Button>
      </div>
    </>
  )
}

export { ReportGeneration }

type StatusModalDialogProps = {
  setOpenStatusModal: (isOpen: boolean) => void
  openStatusModal: boolean
}

const StatusModal: React.FC<StatusModalDialogProps> = ({
  setOpenStatusModal,
  openStatusModal,
}) => {
  const t = useTranslations("Reports.ReportGeneration")

  const { data, error } = useReportGenerationJobStatusQuery({
    pollInterval: POLL_INTERVAL,
  })

  if (!data) return
  if (error) return <div>{t("error", { error: error.message })}</div>

  const startedAt = new Date(
    data.reportGenerationJobStatus.running
      ? data.reportGenerationJobStatus.runStartedAt
      : data.reportGenerationJobStatus.lastRun?.runStartedAt,
  )
  const running = data.reportGenerationJobStatus.running
  const lastRunErrored =
    data.reportGenerationJobStatus.lastRun?.status.toLowerCase() === "failed"

  const rawLog = running
    ? data.reportGenerationJobStatus.logs
    : data.reportGenerationJobStatus.lastRun?.logs
  const cleanLog = rawLog?.replace(/\x1B\[[0-9;]*[A-Za-z]/g, "")

  return (
    <Dialog open={openStatusModal} onOpenChange={setOpenStatusModal}>
      <DialogContent className="w-full max-w-4xl">
        <DialogHeader>
          <DialogTitle>{running ? t("running") : t("detailsOfLastRun")}</DialogTitle>
          <DialogDescription>
            {t("startedAt", { t: formatDate(startedAt, { includeTime: true }) })}
          </DialogDescription>
        </DialogHeader>
        {!running && lastRunErrored && (
          <div>
            <Badge variant="destructive">Errored</Badge>
          </div>
        )}
        <code className="bg-black rounded-md text-white p-2 text-xs h-96 overflow-auto font-mono whitespace-pre">
          {cleanLog}
        </code>
      </DialogContent>
    </Dialog>
  )
}
