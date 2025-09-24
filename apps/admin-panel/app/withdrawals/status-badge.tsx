import { Badge, BadgeProps } from "@lana/web/ui/badge"
import { useTranslations } from "next-intl"

import { WithdrawalStatus } from "@/lib/graphql/generated"

interface StatusBadgeProps extends BadgeProps {
  status: WithdrawalStatus
}

const getVariant = (status: WithdrawalStatus): BadgeProps["variant"] => {
  switch (status) {
    case WithdrawalStatus.PendingApproval:
      return "default"
    case WithdrawalStatus.PendingConfirmation:
      return "default"
    case WithdrawalStatus.Confirmed:
      return "success"
    case WithdrawalStatus.Cancelled:
      return "destructive"
    case WithdrawalStatus.Denied:
      return "destructive"
    case WithdrawalStatus.Reverted:
      return "destructive"
    default: {
      const exhaustiveCheck: never = status
      return exhaustiveCheck
    }
  }
}

export const WithdrawalStatusBadge: React.FC<StatusBadgeProps> = ({
  status,
  ...props
}) => {
  const t = useTranslations("Withdrawals.WithdrawalStatus")
  const variant = getVariant(status)

  return (
    <Badge variant={variant} {...props}>
      {t(status.toLowerCase())}
    </Badge>
  )
}
