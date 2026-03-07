import { Badge } from "@lana/web/ui/badge"
import { useTranslations } from "next-intl"

import { cn } from "@lana/web/utils"
import { BadgeCheck, Clock } from "lucide-react"

import { KycLevel } from "@/lib/graphql/generated"

const getStatusConfig = (level: KycLevel) => {
  switch (level) {
    case KycLevel.Basic:
    case KycLevel.Advanced:
      return {
        icon: BadgeCheck,
        translationKey: "verified",
        className: "text-green-600",
      }
    case KycLevel.NotKyced:
      return {
        icon: Clock,
        translationKey: "noKyc",
        className: "text-muted-foreground",
      }
    default: {
      const exhaustiveCheck: never = level
      return exhaustiveCheck
    }
  }
}

interface KycStatusBadgeProps {
  level: KycLevel | undefined
}

export const KycStatusBadge: React.FC<KycStatusBadgeProps> = ({ level }) => {
  const t = useTranslations("Customers.CustomerDetails.kycStatus")
  if (!level) return null

  const {
    icon: Icon,
    translationKey,
    className: statusClassName,
  } = getStatusConfig(level)

  return (
    <Badge variant="ghost" className={cn("flex items-center gap-1", statusClassName)}>
      <Icon className="h-4 w-4 stroke-[3]" />
      {t(translationKey)}
    </Badge>
  )
}
