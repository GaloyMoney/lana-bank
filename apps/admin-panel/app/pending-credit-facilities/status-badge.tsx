import { Badge, BadgeProps } from "@lana/web/ui/badge"
import { useTranslations } from "next-intl"
import { cn } from "@lana/web/utils"

import { PendingCreditFacilityStatus } from "@/lib/graphql/generated"

const getConfig = (
  status: PendingCreditFacilityStatus,
  t: ReturnType<typeof useTranslations<"PendingCreditFacilities.status">>,
): { label: string; variant: BadgeProps["variant"] } => {
  switch (status) {
    case PendingCreditFacilityStatus.PendingCollateralization:
      return { label: t("pending_collateralization"), variant: "secondary" }
    case PendingCreditFacilityStatus.Completed:
      return { label: t("completed"), variant: "success" }
    default: {
      const exhaustiveCheck: never = status
      return exhaustiveCheck
    }
  }
}

interface PendingCreditFacilityStatusBadgeProps {
  status: PendingCreditFacilityStatus
  plain?: boolean
  className?: string
}

export const PendingCreditFacilityStatusBadge: React.FC<
  PendingCreditFacilityStatusBadgeProps
> = ({ status, plain, className }) => {
  const t = useTranslations("PendingCreditFacilities.status")
  const { label, variant } = getConfig(status, t)
  if (plain) return label

  return (
    <Badge
      variant={variant}
      className={cn(className)}
      data-testid="pending-status-badge"
    >
      {label}
    </Badge>
  )
}
