import { Badge, BadgeProps } from "@/components/primitive/badge"
import { LoanStatus } from "@/lib/graphql/generated"
import { cn } from "@/lib/utils"

interface LoanStatusBadgeProps extends BadgeProps {
  status: LoanStatus
}

const getVariant = (status: LoanStatus) => {
  switch (status) {
    case LoanStatus.Active:
      return "success"
    case LoanStatus.New:
      return "default"
    default:
      return "secondary"
  }
}

export const LoanStatusBadge = ({
  status,
  className,
  ...otherProps
}: LoanStatusBadgeProps) => {
  const variant = getVariant(status)

  return (
    <Badge variant={variant} className={cn(className)} {...otherProps}>
      {status}
    </Badge>
  )
}
