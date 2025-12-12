"use client"

import { Badge, BadgeProps } from "@lana/web/ui/badge"
import { useTranslations } from "next-intl"

type LiquidationStatusBadgeProps = {
  completed: boolean
} & BadgeProps

export const LiquidationStatusBadge = ({
  completed,
  ...badgeProps
}: LiquidationStatusBadgeProps) => {
  const t = useTranslations("Liquidations.status")
  const variant: BadgeProps["variant"] = completed ? "success" : "warning"

  return (
    <Badge variant={variant} {...badgeProps}>
      {completed ? t("completed") : t("inProgress")}
    </Badge>
  )
}
