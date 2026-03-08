"use client"

import { useTranslations } from "next-intl"

import { Badge, BadgeProps } from "@lana/web/ui/badge"

import { DepositAccountStatus } from "@/lib/graphql/generated"

const getConfig = (
  status: DepositAccountStatus,
  t: ReturnType<typeof useTranslations<"DepositAccounts.status">>,
): { label: string; variant: BadgeProps["variant"] } => {
  switch (status) {
    case DepositAccountStatus.Active:
      return { label: t("active"), variant: "success" }
    case DepositAccountStatus.Frozen:
      return { label: t("frozen"), variant: "destructive" }
    case DepositAccountStatus.Closed:
      return { label: t("closed"), variant: "destructive" }
    default: {
      const exhaustiveCheck: never = status
      return exhaustiveCheck
    }
  }
}

export const DepositAccountStatusBadge: React.FC<{
  status: DepositAccountStatus
  plain?: boolean
}> = ({ status, plain }) => {
  const t = useTranslations("DepositAccounts.status")
  const { label, variant } = getConfig(status, t)
  if (plain) return label

  return (
    <Badge data-testid="deposit-account-status-badge" variant={variant}>
      {label}
    </Badge>
  )
}
