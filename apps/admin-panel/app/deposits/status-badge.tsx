import { Badge } from "@lana/web/ui/badge"
import { useTranslations } from "next-intl"

import { DepositStatus } from "@/lib/graphql/generated"

type DepositStatusBadgeProps = {
  status: DepositStatus
  testId?: string
}

export const DepositStatusBadge: React.FC<DepositStatusBadgeProps> = ({
  status,
  testId,
}) => {
  const t = useTranslations("Deposits.DepositStatus")

  const getVariant = (status: DepositStatus) => {
    switch (status) {
      case DepositStatus.Confirmed:
        return "success"
      case DepositStatus.Reverted:
        return "destructive"
      default: {
        const exhaustiveCheck: never = status
        return exhaustiveCheck
      }
    }
  }

  return (
    <Badge variant={getVariant(status)} data-testid={testId}>
      {t(status.toLowerCase())}
    </Badge>
  )
}
