import { Badge, BadgeProps } from "@lana/web/ui/badge"
import { useTranslations } from "next-intl"
import { cn } from "@lana/web/utils"

import { CreditFacilityProposalStatus } from "@/lib/graphql/generated"

const getConfig = (
  status: CreditFacilityProposalStatus,
  t: ReturnType<typeof useTranslations<"CreditFacilityProposals.status">>,
): { label: string; variant: BadgeProps["variant"] } => {
  switch (status) {
    case CreditFacilityProposalStatus.PendingApproval:
      return { label: t("pending_approval"), variant: "secondary" }
    case CreditFacilityProposalStatus.PendingCustomerApproval:
      return { label: t("pending_customer_approval"), variant: "secondary" }
    case CreditFacilityProposalStatus.Approved:
      return { label: t("approved"), variant: "success" }
    case CreditFacilityProposalStatus.Denied:
      return { label: t("denied"), variant: "destructive" }
    case CreditFacilityProposalStatus.CustomerDenied:
      return { label: t("customer_denied"), variant: "destructive" }
    default: {
      const exhaustiveCheck: never = status
      return exhaustiveCheck
    }
  }
}

interface CreditFacilityProposalStatusBadgeProps {
  status: CreditFacilityProposalStatus
  plain?: boolean
  className?: string
}

export const CreditFacilityProposalStatusBadge: React.FC<
  CreditFacilityProposalStatusBadgeProps
> = ({ status, plain, className }) => {
  const t = useTranslations("CreditFacilityProposals.status")
  const { label, variant } = getConfig(status, t)
  if (plain) return label

  return (
    <Badge
      variant={variant}
      className={cn(className)}
      data-testid="proposal-status-badge"
    >
      {label}
    </Badge>
  )
}
