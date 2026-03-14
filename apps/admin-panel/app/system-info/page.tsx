"use client"

import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"
import { LoaderCircle } from "lucide-react"

import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@lana/web/ui/card"

import { useGetBuildInfoQuery, useGetServerTimeQuery } from "@/lib/graphql/generated"
import { env } from "@/env"

gql`
  query GetBuildInfo {
    buildInfo {
      version
      buildProfile
      buildTarget
      enabledFeatures
    }
  }
`

gql`
  query GetServerTime {
    serverTime {
      currentTime
      isArtificial
    }
  }
`

export default function SystemInfoPage() {
  const t = useTranslations("SystemInfo")
  const appVersion = env.NEXT_PUBLIC_APP_VERSION

  const { data, loading } = useGetBuildInfoQuery()
  const buildInfo = data?.buildInfo

  const { data: serverTimeData, loading: serverTimeLoading } = useGetServerTimeQuery({
    pollInterval: 5000,
  })
  const serverTime = serverTimeData?.serverTime

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader>
          <CardTitle>{t("serverTime")}</CardTitle>
        </CardHeader>
        <CardContent>
          {serverTimeLoading ? (
            <LoaderCircle className="animate-spin" />
          ) : serverTime ? (
            <div className="space-y-2">
              <InfoRow label={t("currentTime")} value={serverTime.currentTime} />
              <InfoRow
                label={t("timeMode")}
                value={serverTime.isArtificial ? t("artificial") : t("realtime")}
              />
            </div>
          ) : (
            <p className="text-sm text-muted-foreground">{t("unavailable")}</p>
          )}
        </CardContent>
      </Card>

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
