"use client"

import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"
import { LoaderCircle } from "lucide-react"

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@lana/web/ui/card"
import { formatDate } from "@lana/web/utils"

import {
  type Time,
  useGetBuildInfoQuery,
  useGetTimeQuery,
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
      canAdvanceToNextEndOfDay
    }
  }
`

export default function SystemInfoPage() {
  const t = useTranslations("SystemInfo")
  const appVersion = env.NEXT_PUBLIC_APP_VERSION

  const { data, loading } = useGetBuildInfoQuery()
  const buildInfo = data?.buildInfo
  const {
    data: timeData,
    loading: timeLoading,
    error: timeError,
  } = useGetTimeQuery()

  const time = timeData?.time

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

      <TimeCard time={time} loading={timeLoading} error={timeError} />

      <Card>
        <CardHeader>
          <CardTitle>{t("appConfig")}</CardTitle>
        </CardHeader>
        <CardContent>
          {loading ? (
            <LoaderCircle className="animate-spin" />
          ) : data?.appConfig ? (
            <pre className="text-xs font-mono whitespace-pre-wrap overflow-auto max-h-[600px]">
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

type TimeCardProps = {
  time?: Time
  loading: boolean
  error?: Error
}

function TimeCard({ time, loading, error }: TimeCardProps) {
  const t = useTranslations("SystemInfo")
  const tTime = useTranslations("Configurations.time")

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
    </Card>
  )
}
