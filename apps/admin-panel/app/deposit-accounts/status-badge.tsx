"use client"

import { useTranslations } from "next-intl"

import { Badge } from "@lana/web/ui/badge"

import { DepositAccountStatus } from "@/lib/graphql/generated"

export const DepositAccountStatusBadge: React.FC<{ status: DepositAccountStatus }> = ({
  status,
}) => {
  const t = useTranslations("DepositAccounts.status")

  const getVariant = (status: DepositAccountStatus) => {
    switch (status) {
      case DepositAccountStatus.Active:
        return "success"
      case DepositAccountStatus.Frozen:
        return "destructive"
      case DepositAccountStatus.Inactive:
        return "secondary"
      case DepositAccountStatus.Closed:
        return "destructive"
      default: {
        const exhaustiveCheck: never = status
        return exhaustiveCheck
      }
    }
  }

  return (
    <Badge data-testid="deposit-account-status-badge" variant={getVariant(status)}>
      {t(status.toLowerCase())}
    </Badge>
  )
}
