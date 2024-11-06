import { Badge, BadgeProps } from "@/components/primitive/badge"
import { CreditFacilityStatus } from "@/lib/graphql/generated"
import { cn } from "@/lib/utils"

interface LoanAndCreditFacilityStatusBadgeProps extends BadgeProps {
  status: CreditFacilityStatus
}

const getVariant = (status: CreditFacilityStatus) => {
  switch (status) {
    case CreditFacilityStatus.Active:
      return "success"
    case CreditFacilityStatus.PendingApproval:
      return "default"
    default:
      return "secondary"
  }
}

export const LoanAndCreditFacilityStatusBadge = ({
  status,
  className,
  ...otherProps
}: LoanAndCreditFacilityStatusBadgeProps) => {
  const variant = getVariant(status)

  return (
    <Badge variant={variant} className={cn(className)} {...otherProps}>
      {status.split("_").join(" ")}
    </Badge>
  )
}
