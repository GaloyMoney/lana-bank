"use client"

import { useTranslations } from "next-intl"

import { Badge, BadgeProps } from "@lana/web/ui/badge"

import { ProspectStage } from "@/lib/graphql/generated"

const getConfig = (
  stage: ProspectStage,
  t: ReturnType<typeof useTranslations<"Prospects.stage">>,
): { label: string; variant: BadgeProps["variant"] } => {
  switch (stage) {
    case ProspectStage.New:
      return { label: t("new"), variant: "secondary" }
    case ProspectStage.KycStarted:
      return { label: t("kycStarted"), variant: "warning" }
    case ProspectStage.KycPending:
      return { label: t("kycPending"), variant: "warning" }
    case ProspectStage.KycDeclined:
      return { label: t("kycDeclined"), variant: "destructive" }
    case ProspectStage.Converted:
      return { label: t("converted"), variant: "success" }
    case ProspectStage.Closed:
      return { label: t("closed"), variant: "default" }
    default: {
      const _: never = stage
      return _
    }
  }
}

const ProspectStageBadge = ({
  stage,
  plain,
}: {
  stage: ProspectStage
  plain?: boolean
}) => {
  const t = useTranslations("Prospects.stage")
  const { label, variant } = getConfig(stage, t)
  if (plain) return label
  return <Badge variant={variant}>{label}</Badge>
}

export { ProspectStageBadge }
