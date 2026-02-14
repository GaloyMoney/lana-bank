"use client"

import { useTranslations } from "next-intl"

import { Badge } from "@lana/web/ui/badge"

import { ProspectStage } from "@/lib/graphql/generated"

const ProspectStageBadge = ({ stage }: { stage: ProspectStage }) => {
  const t = useTranslations("Prospects.stage")

  switch (stage) {
    case ProspectStage.New:
      return <Badge variant="secondary">{t("new")}</Badge>
    case ProspectStage.KycStarted:
      return <Badge variant="warning">{t("kycStarted")}</Badge>
    case ProspectStage.KycPending:
      return <Badge variant="warning">{t("kycPending")}</Badge>
    case ProspectStage.KycDeclined:
      return <Badge variant="destructive">{t("kycDeclined")}</Badge>
    case ProspectStage.Converted:
      return <Badge variant="success">{t("converted")}</Badge>
    case ProspectStage.Closed:
      return <Badge variant="default">{t("closed")}</Badge>
    default:
      return <Badge variant="secondary">{stage}</Badge>
  }
}

export { ProspectStageBadge }
