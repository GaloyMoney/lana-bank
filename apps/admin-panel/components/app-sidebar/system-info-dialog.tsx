"use client"

import { useState } from "react"
import { useTranslations } from "next-intl"
import { gql } from "@apollo/client"

import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@lana/web/ui/dialog"

import { useGetBuildInfoQuery } from "@/lib/graphql/generated"

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

interface SystemInfoDialogProps {
  appVersion: string
  children: React.ReactNode
}

export function SystemInfoDialog({ appVersion, children }: SystemInfoDialogProps) {
  const [open, setOpen] = useState(false)
  const t = useTranslations("SystemInfo")

  const { data, loading } = useGetBuildInfoQuery({
    skip: !open,
  })

  const buildInfo = data?.buildInfo

  return (
    <>
      <div onClick={() => setOpen(true)} className="cursor-pointer">
        {children}
      </div>
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>{t("title")}</DialogTitle>
          </DialogHeader>
          <div className="space-y-4">
            <InfoSection title={t("frontend")}>
              <InfoRow label={t("version")} value={appVersion} />
            </InfoSection>

            <InfoSection title={t("backend")}>
              {loading ? (
                <p className="text-sm text-muted-foreground">{t("loading")}</p>
              ) : buildInfo ? (
                <>
                  <InfoRow label={t("version")} value={buildInfo.version} />
                  <InfoRow label={t("buildProfile")} value={buildInfo.buildProfile} />
                  <InfoRow label={t("buildTarget")} value={buildInfo.buildTarget} />
                  {buildInfo.enabledFeatures.length > 0 && (
                    <InfoRow
                      label={t("enabledFeatures")}
                      value={buildInfo.enabledFeatures.join(", ")}
                    />
                  )}
                </>
              ) : (
                <p className="text-sm text-muted-foreground">{t("unavailable")}</p>
              )}
            </InfoSection>
          </div>
        </DialogContent>
      </Dialog>
    </>
  )
}

function InfoSection({
  title,
  children,
}: {
  title: string
  children: React.ReactNode
}) {
  return (
    <div>
      <h3 className="text-sm font-medium mb-2">{title}</h3>
      <div className="rounded-md border p-3 space-y-2">{children}</div>
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
