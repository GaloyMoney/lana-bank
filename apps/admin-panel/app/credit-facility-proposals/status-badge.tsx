import { Badge } from "@lana/web/ui/badge"
import { useTranslations } from "next-intl"
import { cn } from "@lana/web/utils"

import { CreditFacilityProposalStatus } from "@/lib/graphql/generated"

interface CreditFacilityProposalStatusBadgeProps {
  status: CreditFacilityProposalStatus
  className?: string
}

export const CreditFacilityProposalStatusBadge: React.FC<
  CreditFacilityProposalStatusBadgeProps
> = ({ status, className }) => {
  const t = useTranslations("CreditFacilityProposals.status")

  const badgeVariant = () => {
    switch (status) {
      case CreditFacilityProposalStatus.Completed:
        return "success"
      case CreditFacilityProposalStatus.PendngApproval:
        return "secondary"
      case CreditFacilityProposalStatus.PendingCollateralization:
        return "warning"
      default: {
        const exhaustiveCheck: never = status
        return exhaustiveCheck
      }
    }
  }

  return (
    <Badge variant={badgeVariant()} className={cn(className)}>
      {t(status.toLowerCase())}
    </Badge>
  )
}
