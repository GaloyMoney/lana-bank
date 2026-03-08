import { Badge, BadgeProps } from "@lana/web/ui/badge"
import { useTranslations } from "next-intl"

import { DepositStatus } from "@/lib/graphql/generated"

const getConfig = (
  status: DepositStatus,
  t: ReturnType<typeof useTranslations<"Deposits.DepositStatus">>,
): { label: string; variant: BadgeProps["variant"] } => {
  switch (status) {
    case DepositStatus.Confirmed:
      return { label: t("confirmed"), variant: "success" }
    case DepositStatus.Reverted:
      return { label: t("reverted"), variant: "destructive" }
    default: {
      const exhaustiveCheck: never = status
      return exhaustiveCheck
    }
  }
}

type DepositStatusBadgeProps = {
  status: DepositStatus
  plain?: boolean
  testId?: string
}

export const DepositStatusBadge: React.FC<DepositStatusBadgeProps> = ({
  status,
  plain,
  testId,
}) => {
  const t = useTranslations("Deposits.DepositStatus")
  const { label, variant } = getConfig(status, t)
  if (plain) return label

  return (
    <Badge variant={variant} data-testid={testId}>
      {label}
    </Badge>
  )
}
