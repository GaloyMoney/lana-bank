import { Badge } from "@lana/web/ui/badge"
import { useTranslations } from "next-intl"
import { cn } from "@lana/web/utils"

import { PendingCreditFacilityStatus } from "@/lib/graphql/generated"

interface PendingCreditFacilityStatusBadgeProps {
  status: PendingCreditFacilityStatus
  className?: string
}

export const PendingCreditFacilityStatusBadge: React.FC<
  PendingCreditFacilityStatusBadgeProps
> = ({ status, className }) => {
  const t = useTranslations("PendingCreditFacilities.status")

  const badgeVariant = () => {
    switch (status) {
      case PendingCreditFacilityStatus.PendingCollateralization:
        return "secondary"
      case PendingCreditFacilityStatus.Completed:
        return "success"
      default: {
        const exhaustiveCheck: never = status
        return exhaustiveCheck
      }
    }
  }

  return (
    <Badge
      variant={badgeVariant()}
      className={cn(className)}
      data-testid="pending-credit-facility-status-badge"
    >
      {t(status.toLowerCase())}
    </Badge>
  )
}
