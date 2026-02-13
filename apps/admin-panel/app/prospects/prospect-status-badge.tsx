"use client"

import { useTranslations } from "next-intl"

import { Badge } from "@lana/web/ui/badge"

import { ProspectStatus } from "@/lib/graphql/generated"

const ProspectStatusBadge = ({ status }: { status: ProspectStatus }) => {
  const t = useTranslations("Prospects.status")

  switch (status) {
    case ProspectStatus.Open:
      return <Badge variant="secondary">{t("open")}</Badge>
    case ProspectStatus.Converted:
      return <Badge variant="success">{t("converted")}</Badge>
    case ProspectStatus.Closed:
      return <Badge variant="default">{t("closed")}</Badge>
    default:
      return <Badge variant="secondary">{status}</Badge>
  }
}

export { ProspectStatusBadge }
