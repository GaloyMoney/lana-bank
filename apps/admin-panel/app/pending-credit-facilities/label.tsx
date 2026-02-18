import { useTranslations } from "next-intl"
import { Badge } from "@lana/web/ui/badge"

import { PendingCreditFacilityCollateralizationState } from "@/lib/graphql/generated"

interface PendingFacilityCollateralizationStateLabelProps {
  state: PendingCreditFacilityCollateralizationState
}

export const PendingFacilityCollateralizationStateLabel: React.FC<
  PendingFacilityCollateralizationStateLabelProps
> = ({ state }) => {
  const t = useTranslations("PendingCreditFacilities.collateralizationState")

  const badgeVariant = () => {
    switch (state) {
      case PendingCreditFacilityCollateralizationState.FullyCollateralized:
        return "success"
      case PendingCreditFacilityCollateralizationState.UnderCollateralized:
        return "destructive"
      case PendingCreditFacilityCollateralizationState.NotYetCollateralized:
        return "secondary"
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
