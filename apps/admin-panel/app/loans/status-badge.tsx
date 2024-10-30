import { Badge, BadgeProps } from "@/components/primitive/badge"
import { CreditFacilityStatus, LoanStatus } from "@/lib/graphql/generated"
import { cn } from "@/lib/utils"

interface LoanAndCreditFacilityStatusBadgeProps extends BadgeProps {
  status: LoanStatus | CreditFacilityStatus
}

const getVariant = (status: LoanStatus | CreditFacilityStatus) => {
  switch (status) {
    case LoanStatus.Active:
      return "success"
    case LoanStatus.New:
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
