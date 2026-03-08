import { Badge, BadgeProps } from "@lana/web/ui/badge"
import { useTranslations } from "next-intl"

import { WithdrawalStatus } from "@/lib/graphql/generated"

const getConfig = (
  status: WithdrawalStatus,
  t: ReturnType<typeof useTranslations<"Withdrawals.WithdrawalStatus">>,
): { label: string; variant: BadgeProps["variant"] } => {
  switch (status) {
    case WithdrawalStatus.PendingApproval:
      return { label: t("pending_approval"), variant: "default" }
    case WithdrawalStatus.PendingConfirmation:
      return { label: t("pending_confirmation"), variant: "default" }
    case WithdrawalStatus.Confirmed:
      return { label: t("confirmed"), variant: "success" }
    case WithdrawalStatus.Cancelled:
      return { label: t("cancelled"), variant: "destructive" }
    case WithdrawalStatus.Denied:
      return { label: t("denied"), variant: "destructive" }
    case WithdrawalStatus.Reverted:
      return { label: t("reverted"), variant: "destructive" }
    default: {
      const exhaustiveCheck: never = status
      return exhaustiveCheck
    }
  }
}

interface StatusBadgeProps extends BadgeProps {
  status: WithdrawalStatus
  plain?: boolean
}

export const WithdrawalStatusBadge: React.FC<StatusBadgeProps> = ({
  status,
  plain,
  ...props
}) => {
  const t = useTranslations("Withdrawals.WithdrawalStatus")
  const { label, variant } = getConfig(status, t)
  if (plain) return label

  return (
    <Badge variant={variant} {...props}>
      {label}
    </Badge>
  )
}
