import { useTranslations } from "next-intl"

import { CollateralizationState, InterestInterval, Period } from "@/lib/graphql/generated"

export const CollateralizationStateLabel = ({
  state,
}: {
  state: CollateralizationState
}): string => {
  const t = useTranslations("CreditFacilities.collateralizationState")
  if (!state) return ""

  switch (state) {
    case CollateralizationState.FullyCollateralized:
      return t("fullyCollateralized")
    case CollateralizationState.NoCollateral:
      return t("noCollateral")
    case CollateralizationState.NoExposure:
      return t("noExposure")
    case CollateralizationState.UnderLiquidationThreshold:
      return t("underLiquidationThreshold")
    case CollateralizationState.UnderMarginCallThreshold:
      return t("underMarginCallThreshold")
  }
  const exhaustiveCheck: never = state
  return exhaustiveCheck
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
