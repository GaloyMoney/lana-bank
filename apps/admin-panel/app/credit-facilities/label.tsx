import { useTranslations } from "next-intl"
import { Badge } from "@lana/web/ui/badge"

import {
  CollateralizationState,
  InterestInterval,
  Period,
  PendingCreditFacilityCollateralizationState,
} from "@/lib/graphql/generated"

export const CollateralizationStateLabel = ({
  state,
}: {
  state: CollateralizationState
}) => {
  const t = useTranslations("CreditFacilities.collateralizationState")
  if (!state) return null

  const variant = () => {
    switch (state) {
      case CollateralizationState.FullyCollateralized:
        return "success"
      case CollateralizationState.NoCollateral:
        return "secondary"
      case CollateralizationState.NoExposure:
        return "secondary"
      case CollateralizationState.UnderLiquidationThreshold:
        return "destructive"
      case CollateralizationState.UnderMarginCallThreshold:
        return "destructive"
      default:
        return "outline"
    }
  }

  const getText = (): string => {
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

  return <Badge variant={variant()}>{getText()}</Badge>
}

export const PendingFacilityCollateralizationStateLabel = ({
  state,
}: {
  state: PendingCreditFacilityCollateralizationState
}) => {
  const t = useTranslations("CreditFacilities.pendingCollateralizationState")
  if (!state) return null

  const variant = () => {
    switch (state) {
      case PendingCreditFacilityCollateralizationState.FullyCollateralized:
        return "success"
      case PendingCreditFacilityCollateralizationState.UnderCollateralized:
        return "destructive"
      default:
        return "outline"
    }
  }

  const getText = (): string => {
    switch (state) {
      case PendingCreditFacilityCollateralizationState.FullyCollateralized:
        return t("fullyCollateralized")
      case PendingCreditFacilityCollateralizationState.UnderCollateralized:
        return t("underCollateralized")
      case PendingCreditFacilityCollateralizationState.NotYetCollateralized:
        return t("notYetCollateralized")
      default: {
        const exhaustiveCheck: never = state
        return exhaustiveCheck
      }
    }
  }

  return <Badge variant={variant()}>{getText()}</Badge>
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
