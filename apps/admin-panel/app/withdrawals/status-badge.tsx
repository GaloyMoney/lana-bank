import { Badge, BadgeProps } from "@/components/primitive/badge"
import { WithdrawalStatus } from "@/lib/graphql/generated"

interface StatusBadgeProps extends BadgeProps {
  status: WithdrawalStatus
}

const getVariant = (status: WithdrawalStatus) => {
  switch (status) {
    case WithdrawalStatus.Initiated:
      return "default"
    case WithdrawalStatus.Confirmed:
      return "success"
    case WithdrawalStatus.Cancelled:
      return "destructive"
    default:
      return "default"
  }
}

export const WithdrawalStatusBadge: React.FC<StatusBadgeProps> = ({ status }) => {
  const variant = getVariant(status)

  return <Badge variant={variant}>{status}</Badge>
}
