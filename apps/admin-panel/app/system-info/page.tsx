"use client"

import { useState, useEffect } from "react"

import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"
import { LoaderCircle } from "lucide-react"
import { toast } from "sonner"

import { Button } from "@lana/web/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@lana/web/ui/card"
import { formatDate } from "@lana/web/utils"

import {
  type Time,
  EodProcessStatus,
  GetTimeDocument,
  useGetBuildInfoQuery,
  useGetTimeQuery,
  useTimeAdvanceToNextEndOfDayMutation,
} from "@/lib/graphql/generated"
import { env } from "@/env"

gql`
  fragment SystemInfoTimeFields on Time {
    currentDate
    currentTime
    nextEndOfDayAt
    timezone
    endOfDayTime
    canAdvanceToNextEndOfDay
    eodStatus
  }

  query GetBuildInfo {
    buildInfo {
      version
      buildProfile
      buildTarget
      enabledFeatures
    }
    appConfig
  }

  query GetTime {
    time {
      ...SystemInfoTimeFields
    }
  }

  mutation TimeAdvanceToNextEndOfDay {
    timeAdvanceToNextEndOfDay {
      time {
        ...SystemInfoTimeFields
      }
    }
  }
`

export default function SystemInfoPage() {
  const t = useTranslations("SystemInfo")
  const tTime = useTranslations("Configurations.time")
  const appVersion = env.NEXT_PUBLIC_APP_VERSION

  const { data, loading } = useGetBuildInfoQuery()
  const buildInfo = data?.buildInfo
  const {
    data: timeData,
    loading: timeLoading,
    error: timeError,
  } = useGetTimeQuery()

  const time = timeData?.time

  const [advanceToNextEndOfDay, { loading: advanceLoading }] =
    useTimeAdvanceToNextEndOfDayMutation({
      update(cache, { data: mutationData }) {
        if (mutationData?.timeAdvanceToNextEndOfDay?.time) {
          cache.writeQuery({
            query: GetTimeDocument,
            data: { time: mutationData.timeAdvanceToNextEndOfDay.time },
          })
        }
      },
    })

  const handleAdvanceToNextEndOfDay = async () => {
    try {
      await advanceToNextEndOfDay()
      toast.success(tTime("advanceSuccess"))
    } catch (err) {
      if (err instanceof Error && err.message) {
        toast.error(tTime("advanceErrorWithReason", { error: err.message }))
      } else {
        toast.error(tTime("advanceError"))
      }
    }
  }

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader>
          <CardTitle>{t("frontend")}</CardTitle>
        </CardHeader>
        <CardContent>
          <InfoRow label={t("version")} value={appVersion || "0.0.0-dev"} />
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>{t("backend")}</CardTitle>
        </CardHeader>
        <CardContent>
          {loading ? (
            <LoaderCircle className="animate-spin" />
          ) : buildInfo ? (
            <div className="space-y-2">
              <InfoRow label={t("version")} value={buildInfo.version} />
              <InfoRow label={t("buildProfile")} value={buildInfo.buildProfile} />
              <InfoRow label={t("buildTarget")} value={buildInfo.buildTarget} />
              {buildInfo.enabledFeatures.length > 0 && (
                <InfoRow
                  label={t("enabledFeatures")}
                  value={buildInfo.enabledFeatures.join(", ")}
                />
              )}
            </div>
          ) : (
            <p className="text-sm text-muted-foreground">{t("unavailable")}</p>
          )}
        </CardContent>
      </Card>

      <TimeCard
        time={time}
        loading={timeLoading}
        error={timeError}
        onAdvance={handleAdvanceToNextEndOfDay}
        advanceLoading={advanceLoading}
      />

      <Card>
        <CardHeader>
          <CardTitle>{t("appConfig")}</CardTitle>
        </CardHeader>
        <CardContent>
          {loading ? (
            <LoaderCircle className="animate-spin" />
          ) : data?.appConfig ? (
            <pre className="text-xs font-mono whitespace-pre overflow-x-auto rounded-md bg-muted p-4">
              {data.appConfig}
            </pre>
          ) : (
            <p className="text-sm text-muted-foreground">{t("unavailable")}</p>
          )}
        </CardContent>
      </Card>
    </div>
  )
}

function InfoRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex justify-between text-sm">
      <span className="text-muted-foreground">{label}</span>
      <span className="font-mono text-xs">{value}</span>
    </div>
  )
}

function useBrowserTime() {
  const [now, setNow] = useState<string>("")
  useEffect(() => {
    const update = () => setNow(new Date().toLocaleString())
    update()
    const id = setInterval(update, 1000)
    return () => clearInterval(id)
  }, [])
  return now
}

const EOD_IN_PROGRESS_STATUSES: EodProcessStatus[] = [
  EodProcessStatus.Initialized,
  EodProcessStatus.AwaitingPhase1,
  EodProcessStatus.Phase1Complete,
  EodProcessStatus.AwaitingPhase2,
]

function isEodInProgress(status: EodProcessStatus | null | undefined): boolean {
  return status != null && EOD_IN_PROGRESS_STATUSES.includes(status)
}

function formatEodStatus(status: EodProcessStatus): string {
  const labels: Record<EodProcessStatus, string> = {
    [EodProcessStatus.Initialized]: "Initialized",
    [EodProcessStatus.AwaitingPhase1]: "Awaiting Phase 1",
    [EodProcessStatus.Phase1Complete]: "Phase 1 Complete",
    [EodProcessStatus.AwaitingPhase2]: "Awaiting Phase 2",
    [EodProcessStatus.Completed]: "Completed",
    [EodProcessStatus.Failed]: "Failed",
    [EodProcessStatus.Cancelled]: "Cancelled",
  }
  return labels[status]
}

type TimeCardProps = {
  time?: Time
  loading: boolean
  error?: Error
  onAdvance: () => void
  advanceLoading: boolean
}

function TimeCard({ time, loading, error, onAdvance, advanceLoading }: TimeCardProps) {
  const t = useTranslations("SystemInfo")
  const tTime = useTranslations("Configurations.time")
  const browserTime = useBrowserTime()

  return (
    <Card>
      <CardHeader>
        <CardTitle>{tTime("title")}</CardTitle>
        <CardDescription>{tTime("description")}</CardDescription>
      </CardHeader>
      <CardContent>
        {loading ? (
          <LoaderCircle className="animate-spin" />
        ) : error ? (
          <p className="text-sm text-destructive">{tTime("loadError")}</p>
        ) : time ? (
          <div className="space-y-2">
            <InfoRow
              label={tTime("currentDate")}
              value={formatDate(time.currentDate, { includeTime: false })}
            />
            <InfoRow
              label={tTime("currentTime")}
              value={formatDate(time.currentTime)}
            />
            <InfoRow label={tTime("browserTime")} value={browserTime} />
            <InfoRow label={tTime("timezone")} value={time.timezone} />
            <InfoRow label={tTime("endOfDayTime")} value={time.endOfDayTime} />
            <InfoRow
              label={tTime("nextEndOfDay")}
              value={formatDate(time.nextEndOfDayAt)}
            />
            <InfoRow
              label={t("timeMode")}
              value={time.canAdvanceToNextEndOfDay ? t("manual") : t("realtime")}
            />
            {time.eodStatus && (
              <InfoRow label={tTime("eodStatus")} value={formatEodStatus(time.eodStatus)} />
            )}
          </div>
        ) : null}
      </CardContent>
      {time?.canAdvanceToNextEndOfDay && (
        <CardFooter>
          <Button onClick={onAdvance} disabled={advanceLoading || isEodInProgress(time.eodStatus)}>
            {advanceLoading ? (
              <LoaderCircle className="animate-spin mr-2 h-4 w-4" />
            ) : null}
            {tTime("advanceToNextEndOfDay")}
          </Button>
        </CardFooter>
      )}
    </Card>
  )
}
