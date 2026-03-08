import { useTranslations } from "next-intl"

import { CollateralDirection } from "@/lib/graphql/generated"

export const useCollateralDirectionLabel = () => {
  const t = useTranslations("Common.formatCollateralDirection")
  return (collateralDirection: CollateralDirection) => {
    switch (collateralDirection) {
      case CollateralDirection.Add:
        return t("added")
      case CollateralDirection.Remove:
        return t("removed")
      default: {
        const exhaustiveCheck: never = collateralDirection
        return exhaustiveCheck
      }
    }
  }
}
