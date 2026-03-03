"use client"

import { Badge, BadgeProps } from "@lana/web/ui/badge"
import { useTranslations } from "next-intl"

type LiquidationStatusBadgeProps = {
  completed: boolean
  plain?: boolean
} & BadgeProps

export const LiquidationStatusBadge = ({
  completed,
  plain,
  ...badgeProps
}: LiquidationStatusBadgeProps) => {
  const t = useTranslations("Liquidations.status")
  const label = completed ? t("completed") : t("inProgress")
  if (plain) return label
  const variant: BadgeProps["variant"] = completed ? "success" : "warning"

  return (
    <Badge variant={variant} {...badgeProps}>
      {label}
    </Badge>
  )
}
