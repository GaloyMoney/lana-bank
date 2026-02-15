import { Badge } from "@lana/web/ui/badge"
import { useTranslations } from "next-intl"

import { cn } from "@lana/web/utils"
import { BadgeCheck, Clock, CircleX } from "lucide-react"

import { KycVerification } from "@/lib/graphql/generated"

const getStatusConfig = (status: KycVerification) => {
  switch (status) {
    case KycVerification.Verified:
      return {
        icon: BadgeCheck,
        translationKey: "verified",
        className: "text-green-600",
      }
    case KycVerification.NoKyc:
      return {
        icon: Clock,
        translationKey: "noKyc",
        className: "text-muted-foreground",
      }
    case KycVerification.Rejected:
      return {
        icon: CircleX,
        translationKey: "rejected",
        className: "text-destructive",
      }
    default: {
      const exhaustiveCheck: never = status
      return exhaustiveCheck
    }
  }
}

interface KycStatusBadgeProps {
  status: KycVerification | undefined
}

export const KycStatusBadge: React.FC<KycStatusBadgeProps> = ({ status }) => {
  const t = useTranslations("Customers.CustomerDetails.kycStatus")
  if (!status) return null

  const {
    icon: Icon,
    translationKey,
    className: statusClassName,
  } = getStatusConfig(status)

  return (
    <Badge variant="ghost" className={cn("flex items-center gap-1", statusClassName)}>
      <Icon className="h-4 w-4 stroke-[3]" />
      {t(translationKey)}
    </Badge>
  )
}
