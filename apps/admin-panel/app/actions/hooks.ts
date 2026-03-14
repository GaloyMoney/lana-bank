import { useTranslations } from "next-intl"

import { ApprovalProcessType, ApprovalRules } from "@/lib/graphql/generated"

export const useProcessTypeLabel = () => {
  const t = useTranslations("Common.formatProcessType")
  return (processType: ApprovalProcessType) => {
    switch (processType) {
      case ApprovalProcessType.CreditFacilityProposalApproval:
        return t("creditFacilityProposal")
      case ApprovalProcessType.WithdrawalApproval:
        return t("withdrawal")
      case ApprovalProcessType.DisbursalApproval:
        return t("disbursal")
      default: {
        const exhaustiveCheck: never = processType
        return exhaustiveCheck
      }
    }
  }
}

export const useRuleLabel = () => {
  const t = useTranslations("Common.formatRule")
  return (rule: ApprovalRules | null | undefined): string => {
    if (!rule || !rule.__typename) {
      return t("noRulesDefined")
    }

    const { __typename } = rule
    switch (__typename) {
      case "CommitteeApproval":
        return t("allMembersMustApprove")
      case "AutoApproval":
        return rule.autoApprove ? t("systemAutoApproval") : t("systemManualApproval")
      default: {
        const exhaustiveCheck: never = __typename
        return exhaustiveCheck
      }
    }
  }
}
