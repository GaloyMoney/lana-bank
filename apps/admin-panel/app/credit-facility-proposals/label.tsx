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

  const variant = (): "default" | "secondary" | "destructive" | "outline" => {
    switch (state) {
      case CreditFacilityProposalCollateralizationState.FullyCollateralized:
        return "default"
      case CreditFacilityProposalCollateralizationState.UnderCollateralized:
        return "destructive"
      default: {
        const exhaustiveCheck: never = state
        return exhaustiveCheck
      }
    }
  }

  return (
    <Badge variant={variant()}>
      {t(state.toLowerCase().replace(/_/g, ""))}
    </Badge>
  )
}