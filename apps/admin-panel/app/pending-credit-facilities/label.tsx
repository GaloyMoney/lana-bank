import { useTranslations } from "next-intl"
import { Badge } from "@lana/web/ui/badge"

import { PendingCreditFacilityCollateralizationState } from "@/lib/graphql/generated"

interface PendingCreditFacilityCollateralizationStateLabelProps {
  state: PendingCreditFacilityCollateralizationState
}

export const PendingCreditFacilityCollateralizationStateLabel: React.FC<
  PendingCreditFacilityCollateralizationStateLabelProps
> = ({ state }) => {
  const t = useTranslations("PendingCreditFacilities.collateralizationState")

  const badgeVariant = () => {
    switch (state) {
      case PendingCreditFacilityCollateralizationState.FullyCollateralized:
        return "success"
      case PendingCreditFacilityCollateralizationState.UnderCollateralized:
        return "destructive"
      default: {
        const exhaustiveCheck: never = state
        return exhaustiveCheck
      }
    }
  }

  return (
    <Badge variant={badgeVariant()} data-testid="collateralization-state-label">
      {t(state.toLowerCase())}
    </Badge>
  )
}
