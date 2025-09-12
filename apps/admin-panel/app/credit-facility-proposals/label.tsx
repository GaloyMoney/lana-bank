import { useTranslations } from "next-intl"
import { Badge } from "@lana/web/ui/badge"

import { CreditFacilityProposalCollateralizationState } from "@/lib/graphql/generated"

interface CreditFacilityProposalCollateralizationStateLabelProps {
  state: CreditFacilityProposalCollateralizationState
}

export const CreditFacilityProposalCollateralizationStateLabel: React.FC<
  CreditFacilityProposalCollateralizationStateLabelProps
> = ({ state }) => {
  const t = useTranslations("CreditFacilityProposals.collateralizationState")

  const variant = () => {
    switch (state) {
      case CreditFacilityProposalCollateralizationState.FullyCollateralized:
        return "success"
      case CreditFacilityProposalCollateralizationState.UnderCollateralized:
        return "warning"
      default: {
        const exhaustiveCheck: never = state
        return exhaustiveCheck
      }
    }
  }

  return <Badge variant={variant()}>{t(state.toLowerCase().replace(/_/g, ""))}</Badge>
}
