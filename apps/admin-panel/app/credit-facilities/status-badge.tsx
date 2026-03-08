import { Badge, BadgeProps } from "@lana/web/ui/badge"
import { useTranslations } from "next-intl"

import { CreditFacilityStatus } from "@/lib/graphql/generated"
import { cn } from "@/lib/utils"

const getConfig = (
  status: CreditFacilityStatus,
  t: ReturnType<typeof useTranslations<"CreditFacilities.CreditFacilityStatus">>,
): { label: string; variant: BadgeProps["variant"] } => {
  switch (status) {
    case CreditFacilityStatus.Active:
      return { label: t("active"), variant: "success" }
    case CreditFacilityStatus.Closed:
      return { label: t("closed"), variant: "secondary" }
    case CreditFacilityStatus.Matured:
      return { label: t("matured"), variant: "secondary" }
    default: {
      const exhaustiveCheck: never = status
      return exhaustiveCheck
    }
  }
}

interface LoanAndCreditFacilityStatusBadgeProps extends BadgeProps {
  status: CreditFacilityStatus
  plain?: boolean
}

export const LoanAndCreditFacilityStatusBadge = ({
  status,
  plain,
  className,
  ...otherProps
}: LoanAndCreditFacilityStatusBadgeProps) => {
  const t = useTranslations("CreditFacilities.CreditFacilityStatus")
  const { label, variant } = getConfig(status, t)
  if (plain) return label

  return (
    <Badge variant={variant} className={cn(className)} {...otherProps}>
      {label}
    </Badge>
  )
}
