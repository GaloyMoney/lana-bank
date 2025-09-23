import { Badge, BadgeProps } from "@lana/web/ui/badge"
import { useTranslations } from "next-intl"

import { DisbursalStatus } from "@/lib/graphql/generated"

interface StatusBadgeProps extends BadgeProps {
  status: DisbursalStatus
}

const getVariant = (status: DisbursalStatus): BadgeProps["variant"] => {
  switch (status) {
    case DisbursalStatus.New:
      return "default"
    case DisbursalStatus.Approved:
      return "default"
    case DisbursalStatus.Confirmed:
      return "success"
    case DisbursalStatus.Denied:
      return "destructive"
    default:
      return "default"
  }
}

export const DisbursalStatusBadge: React.FC<StatusBadgeProps> = ({
  status,
  ...props
}) => {
  const t = useTranslations("Disbursals.DisbursalStatus")
  const variant = getVariant(status)

  const getTranslatedStatus = (status: DisbursalStatus): string => {
    switch (status) {
      case DisbursalStatus.New:
        return t("new", { defaultMessage: "NEW" })
      case DisbursalStatus.Approved:
        return t("approved", { defaultMessage: "APPROVED" })
      case DisbursalStatus.Confirmed:
        return t("confirmed", { defaultMessage: "CONFIRMED" })
      case DisbursalStatus.Denied:
        return t("denied", { defaultMessage: "DENIED" })
      default:
        return String(status)
    }
  }

  return (
    <Badge variant={variant} {...props}>
      {getTranslatedStatus(status)}
    </Badge>
  )
}
