"use client"

import { useTranslations } from "next-intl"

import { Badge } from "@lana/web/ui/badge"

import { KycStatus } from "@/lib/graphql/generated"

const KycStatusBadge = ({ status }: { status: KycStatus }) => {
  const t = useTranslations("Prospects.kycStatus")

  switch (status) {
    case KycStatus.Approved:
      return <Badge variant="success">{t("approved")}</Badge>
    case KycStatus.Started:
      return <Badge variant="warning">{t("started")}</Badge>
    case KycStatus.Pending:
      return <Badge variant="warning">{t("pending")}</Badge>
    case KycStatus.Declined:
      return <Badge variant="destructive">{t("declined")}</Badge>
    case KycStatus.NotStarted:
      return <Badge variant="secondary">{t("notStarted")}</Badge>
    default:
      return <Badge variant="secondary">{status}</Badge>
  }
}

export { KycStatusBadge }
