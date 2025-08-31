"use client"

import { useTranslations } from "next-intl"

import { Layer } from "@/lib/graphql/generated"

const LayerLabel = ({ value }: { value: Layer }) => {
  const t = useTranslations("Journal.layer")

  const label = ((layer: Layer): string => {
    switch (layer) {
      case Layer.Settled:
        return t("SETTLED")
      case Layer.Pending:
        return t("PENDING")
      case Layer.Encumbrance:
        return t("ENCUMBRANCE")
      default: {
        const exhaustiveCheck: never = layer
        return exhaustiveCheck
      }
    }
  })(value)

  return <span>{label}</span>
}

export default LayerLabel
