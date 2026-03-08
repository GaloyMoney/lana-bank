import { useTranslations } from "next-intl"
import { Badge, BadgeProps } from "@lana/web/ui/badge"

import {
  CollateralizationState,
  InterestInterval,
  Period,
} from "@/lib/graphql/generated"

const getCollateralizationConfig = (
  state: CollateralizationState,
  t: ReturnType<typeof useTranslations<"CreditFacilities.collateralizationState">>,
): { label: string; variant: BadgeProps["variant"] } => {
  switch (state) {
    case CollateralizationState.FullyCollateralized:
      return { label: t("fullyCollateralized"), variant: "success" }
    case CollateralizationState.NoCollateral:
      return { label: t("noCollateral"), variant: "secondary" }
    case CollateralizationState.NoExposure:
      return { label: t("noExposure"), variant: "secondary" }
    case CollateralizationState.UnderLiquidationThreshold:
      return { label: t("underLiquidationThreshold"), variant: "destructive" }
    case CollateralizationState.UnderMarginCallThreshold:
      return { label: t("underMarginCallThreshold"), variant: "destructive" }
    default: {
      const exhaustiveCheck: never = state
      return exhaustiveCheck
    }
  }
}

export const CollateralizationStateLabel = ({
  state,
  plain,
}: {
  state: CollateralizationState
  plain?: boolean
}) => {
  const t = useTranslations("CreditFacilities.collateralizationState")
  if (!state) return null

  const { label, variant } = getCollateralizationConfig(state, t)
  if (plain) return label

  return <Badge variant={variant}>{label}</Badge>
}

export const InterestIntervalLabel = ({
  interval,
}: {
  interval: InterestInterval
}): string => {
  const t = useTranslations("interestInterval")
  if (!interval) return ""

  switch (interval) {
    case InterestInterval.EndOfDay:
      return t("endOfDay")
    case InterestInterval.EndOfMonth:
      return t("endOfMonth")
  }

  const exhaustiveCheck: never = interval
  return exhaustiveCheck
}

export const PeriodLabel = ({ period }: { period: Period }): string => {
  const t = useTranslations("period")
  if (!period) return ""

  switch (period) {
    case Period.Days:
      return t("days")
    case Period.Months:
      return t("months")
  }
  const exhaustiveCheck: never = period
  return exhaustiveCheck
}
