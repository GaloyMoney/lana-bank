import { useTranslations } from "next-intl"
import { Badge } from "@lana/web/ui/badge"

import { PendingCreditFacilityCollateralizationState } from "@/lib/graphql/generated"

interface CreditFacilityProposalCollateralizationStateLabelProps {
  state: PendingCreditFacilityCollateralizationState
}

export const CreditFacilityProposalCollateralizationStateLabel: React.FC<
  CreditFacilityProposalCollateralizationStateLabelProps
> = ({ state }) => {
  const t = useTranslations("CreditFacilityProposals.collateralizationState")

  const variant = () => {
    switch (state) {
      case PendingCreditFacilityCollateralizationState.FullyCollateralized:
        return "success"
      case PendingCreditFacilityCollateralizationState.UnderCollateralized:
        return "warning"
      case PendingCreditFacilityCollateralizationState.NotYetCollateralized:
        return "secondary"
      default: {
        const exhaustiveCheck: never = state
        return exhaustiveCheck
      }
    }
  }

  return <Badge variant={variant()}>{t(state.toLowerCase().replace(/_/g, ""))}</Badge>
}
