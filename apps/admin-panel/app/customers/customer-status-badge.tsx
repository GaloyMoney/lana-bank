import { Badge } from "@lana/web/ui/badge"
import { useTranslations } from "next-intl"

import { CustomerStatus } from "@/lib/graphql/generated"

const getStatusConfig = (status: CustomerStatus) => {
  switch (status) {
    case CustomerStatus.Active:
      return {
        translationKey: "active",
        variant: "success" as const,
      }
    case CustomerStatus.Frozen:
      return {
        translationKey: "frozen",
        variant: "destructive" as const,
      }
    default: {
      const exhaustiveCheck: never = status
      return exhaustiveCheck
    }
  }
}

interface CustomerStatusBadgeProps {
  status: CustomerStatus | undefined
}

export const CustomerStatusBadge: React.FC<CustomerStatusBadgeProps> = ({ status }) => {
  const t = useTranslations("Customers.CustomerDetails.details.customerStatus")
  if (!status) return null

  const { translationKey, variant } = getStatusConfig(status)

  return (
    <Badge variant={variant} className="w-fit">
      {t(translationKey)}
    </Badge>
  )
}
