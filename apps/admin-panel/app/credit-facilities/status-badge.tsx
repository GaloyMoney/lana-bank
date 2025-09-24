import { Badge, BadgeProps } from "@lana/web/ui/badge"
import { useTranslations } from "next-intl"

import { CreditFacilityStatus } from "@/lib/graphql/generated"
import { cn } from "@/lib/utils"

interface LoanAndCreditFacilityStatusBadgeProps extends BadgeProps {
  status: CreditFacilityStatus
}

const getVariant = (status: CreditFacilityStatus): BadgeProps["variant"] => {
  switch (status) {
    case CreditFacilityStatus.Active:
      return "success"
    case CreditFacilityStatus.Closed:
      return "secondary"
    case CreditFacilityStatus.Matured:
      return "secondary"
    default: {
      const exhaustiveCheck: never = status
      return exhaustiveCheck
    }
  }
}

export const LoanAndCreditFacilityStatusBadge = ({
  status,
  className,
  ...otherProps
}: LoanAndCreditFacilityStatusBadgeProps) => {
  const t = useTranslations("CreditFacilities.CreditFacilityStatus")
  const variant = getVariant(status)

  const getTranslatedStatus = (status: CreditFacilityStatus): string => {
    switch (status) {
      case CreditFacilityStatus.Active:
        return t("active")
      case CreditFacilityStatus.Closed:
        return t("closed")
      case CreditFacilityStatus.Matured:
        return t("matured")
      default: {
        const exhaustiveCheck: never = status
        return exhaustiveCheck
      }
    }
  }

  return (
    <Badge variant={variant} className={cn(className)} {...otherProps}>
      {getTranslatedStatus(status)}
    </Badge>
  )
}
