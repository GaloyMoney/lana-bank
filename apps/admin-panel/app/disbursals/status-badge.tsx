import { Badge, BadgeProps } from "@lana/web/ui/badge"
import { useTranslations } from "next-intl"

import { DisbursalStatus } from "@/lib/graphql/generated"

const getConfig = (
  status: DisbursalStatus,
  t: ReturnType<typeof useTranslations<"Disbursals.DisbursalStatus">>,
): { label: string; variant: BadgeProps["variant"] } => {
  switch (status) {
    case DisbursalStatus.New:
      return { label: t("new", { defaultMessage: "NEW" }), variant: "default" }
    case DisbursalStatus.Approved:
      return { label: t("approved", { defaultMessage: "APPROVED" }), variant: "default" }
    case DisbursalStatus.Confirmed:
      return { label: t("confirmed", { defaultMessage: "CONFIRMED" }), variant: "success" }
    case DisbursalStatus.Denied:
      return { label: t("denied", { defaultMessage: "DENIED" }), variant: "destructive" }
    default: {
      const exhaustiveCheck: never = status
      return exhaustiveCheck
    }
  }
}

interface StatusBadgeProps extends BadgeProps {
  status: DisbursalStatus
  plain?: boolean
}

export const DisbursalStatusBadge: React.FC<StatusBadgeProps> = ({
  status,
  plain,
  ...props
}) => {
  const t = useTranslations("Disbursals.DisbursalStatus")
  const { label, variant } = getConfig(status, t)
  if (plain) return label

  return (
    <Badge variant={variant} {...props}>
      {label}
    </Badge>
  )
}
