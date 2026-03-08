import { useTranslations } from "next-intl"
import { Badge, BadgeProps } from "@lana/web/ui/badge"

import { PendingCreditFacilityCollateralizationState } from "@/lib/graphql/generated"

const getConfig = (
  state: PendingCreditFacilityCollateralizationState,
  t: ReturnType<
    typeof useTranslations<"PendingCreditFacilities.collateralizationState">
  >,
): { label: string; variant: BadgeProps["variant"] } => {
  switch (state) {
    case PendingCreditFacilityCollateralizationState.FullyCollateralized:
      return { label: t("fully_collateralized"), variant: "success" }
    case PendingCreditFacilityCollateralizationState.UnderCollateralized:
      return { label: t("under_collateralized"), variant: "destructive" }
    case PendingCreditFacilityCollateralizationState.NotYetCollateralized:
      return { label: t("not_yet_collateralized"), variant: "secondary" }
    default: {
      const exhaustiveCheck: never = state
      return exhaustiveCheck
    }
  }
}

interface PendingFacilityCollateralizationStateLabelProps {
  state: PendingCreditFacilityCollateralizationState
  plain?: boolean
}

export const PendingFacilityCollateralizationStateLabel: React.FC<
  PendingFacilityCollateralizationStateLabelProps
> = ({ state, plain }) => {
  const t = useTranslations("PendingCreditFacilities.collateralizationState")
  const { label, variant } = getConfig(state, t)
  if (plain) return label

  return (
    <Badge variant={variant} data-testid="collateralization-state-label">
      {label}
    </Badge>
  )
}
