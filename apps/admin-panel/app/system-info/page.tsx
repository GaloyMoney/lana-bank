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
  GetTimeDocument,
  type Time,
  useGetBuildInfoQuery,
  useGetTimeQuery,
  useTimeAdvanceToNextEndOfDayMutation,
} from "@/lib/graphql/generated"
import { env } from "@/env"

gql`
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
      currentDate
      currentTime
      nextEndOfDayAt
      timezone
      endOfDayTime
      canAdvanceToNextEndOfDay
    }
  }

  mutation TimeAdvanceToNextEndOfDay {
    timeAdvanceToNextEndOfDay {
      time {
        currentDate
        currentTime
        nextEndOfDayAt
        timezone
        endOfDayTime
        canAdvanceToNextEndOfDay
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
  const [advanceTime, { loading: timeAdvanceLoading }] =
    useTimeAdvanceToNextEndOfDayMutation({
      update(cache, { data }) {
        const time = data?.timeAdvanceToNextEndOfDay.time

        if (!time) {
          return
        }

        cache.writeQuery({
          query: GetTimeDocument,
          data: {
            time,
          },
        })
      },
    })

  const time = timeData?.time

  const handleAdvanceToNextEndOfDay = async () => {
    try {
      const result = await advanceTime()
      const updated = result.data?.timeAdvanceToNextEndOfDay.time

      if (!updated) {
        toast.error(tTime("advanceError"))
        return
      }

      toast.success(tTime("advanceSuccess"))
    } catch (error) {
      console.error("Failed to advance time:", error)

      const errorMessage = error instanceof Error ? error.message : null

      toast.error(
        errorMessage
          ? tTime("advanceErrorWithReason", { error: errorMessage })
          : tTime("advanceError"),
      )
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
        advanceLoading={timeAdvanceLoading}
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

type TimeCardProps = {
  time?: Time
  loading: boolean
  error?: Error
  onAdvance: () => Promise<void>
  advanceLoading: boolean
}

function TimeCard({
  time,
  loading,
  error,
  onAdvance,
  advanceLoading,
}: TimeCardProps) {
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
              label={t("timeMode")}
              value={time.canAdvanceToNextEndOfDay ? t("simulated") : t("realtime")}
            />
            <InfoRow
              label={tTime("nextEndOfDay")}
              value={formatDate(time.nextEndOfDayAt)}
            />
          </div>
        ) : null}
      </CardContent>
      {time?.canAdvanceToNextEndOfDay && (
        <CardFooter className="justify-end">
          <Button
            onClick={() => {
              onAdvance().catch(() => undefined)
            }}
            loading={advanceLoading}
            data-testid="advance-time"
          >
            {tTime("advanceToNextEndOfDay")}
          </Button>
        </CardFooter>
      )}
    </Card>
  )
}
