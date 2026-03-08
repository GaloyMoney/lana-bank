import { Badge, BadgeProps } from "@lana/web/ui/badge"
import { useTranslations } from "next-intl"

import { CustomerStatus } from "@/lib/graphql/generated"

const getConfig = (
  status: CustomerStatus,
  t: ReturnType<typeof useTranslations<"Customers.CustomerDetails.details.customerStatus">>,
): { label: string; variant: BadgeProps["variant"] } => {
  switch (status) {
    case CustomerStatus.Active:
      return { label: t("active"), variant: "success" }
    case CustomerStatus.Frozen:
      return { label: t("frozen"), variant: "destructive" }
    case CustomerStatus.Closed:
      return { label: t("closed"), variant: "secondary" }
    default: {
      const exhaustiveCheck: never = status
      return exhaustiveCheck
    }
  }
}

interface CustomerStatusBadgeProps {
  status: CustomerStatus | undefined
  plain?: boolean
}

export const CustomerStatusBadge: React.FC<CustomerStatusBadgeProps> = ({
  status,
  plain,
}) => {
  const t = useTranslations("Customers.CustomerDetails.details.customerStatus")
  if (!status) return null

  const { label, variant } = getConfig(status, t)
  if (plain) return label

  return (
    <Badge variant={variant} className="w-fit">
      {label}
    </Badge>
  )
}
