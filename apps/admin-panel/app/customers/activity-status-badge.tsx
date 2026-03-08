import { Badge, BadgeProps } from "@lana/web/ui/badge"
import { useTranslations } from "next-intl"

import { Activity } from "@/lib/graphql/generated"

const getConfig = (
  status: Activity,
  t: ReturnType<typeof useTranslations<"Customers.status">>,
): { label: string; variant: BadgeProps["variant"] } => {
  switch (status) {
    case Activity.Active:
      return { label: t("active"), variant: "success" }
    case Activity.Inactive:
      return { label: t("inactive"), variant: "secondary" }
    case Activity.Escheatable:
      return { label: t("escheatable"), variant: "destructive" }
    default: {
      const exhaustiveCheck: never = status
      return exhaustiveCheck
    }
  }
}

interface ActivityStatusBadgeProps {
  status: Activity | undefined
  plain?: boolean
}

export const ActivityStatusBadge: React.FC<ActivityStatusBadgeProps> = ({
  status,
  plain,
}) => {
  const t = useTranslations("Customers.status")
  if (!status) return null

  const { label, variant } = getConfig(status, t)
  if (plain) return label

  return (
    <Badge variant={variant} className="w-fit">
      {label}
    </Badge>
  )
}
