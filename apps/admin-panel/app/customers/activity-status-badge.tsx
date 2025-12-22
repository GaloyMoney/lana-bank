import { Badge } from "@lana/web/ui/badge"
import { useTranslations } from "next-intl"

import { Activity } from "@/lib/graphql/generated"

const getStatusConfig = (status: Activity) => {
  switch (status) {
    case Activity.Active:
      return {
        translationKey: "active",
        variant: "success" as const,
      }
    case Activity.Inactive:
      return {
        translationKey: "inactive",
        variant: "secondary" as const,
      }
    case Activity.Suspended:
      return {
        translationKey: "suspended",
        variant: "destructive" as const,
      }
    default: {
      const exhaustiveCheck: never = status
      return exhaustiveCheck
    }
  }
}

interface ActivityStatusBadgeProps {
  status: Activity | undefined
}

export const ActivityStatusBadge: React.FC<ActivityStatusBadgeProps> = ({ status }) => {
  const t = useTranslations("Customers.CustomerDetails.details.status")
  if (!status) return null

  const { translationKey, variant } = getStatusConfig(status)

  return (
    <Badge variant={variant} className="w-fit">
      {t(translationKey)}
    </Badge>
  )
}
