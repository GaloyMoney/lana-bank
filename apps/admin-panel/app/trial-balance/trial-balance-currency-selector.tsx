"use client"
import React from "react"
import { useTranslations } from "next-intl"
import { Label } from "@lana/web/ui/label"
import { RadioGroup, RadioGroupItem } from "@lana/web/ui/radio-group"

import { Currency } from "@/components/balance/balance"

export type TrialBalanceLayers = "settled" | "pending"

interface CurrencySelectionProps {
  currency: Currency
  setCurrency: (currency: Currency) => void
}

export const TrialBalanceCurrencySelection: React.FC<CurrencySelectionProps> = ({
  currency,
  setCurrency,
}) => {
  const t = useTranslations("CurrencyLayerSelection")
  return (
    <div>
      <Label>{t("currency.label")}</Label>
      <RadioGroup
        className="flex items-center py-2.5 bg-muted p-2 rounded-md"
        value={currency}
        onValueChange={(v: Currency) => setCurrency(v)}
      >
        <div className="flex items-center space-x-1 mr-4">
          <RadioGroupItem value="usd" id="currency-usd" />
          <Label htmlFor="currency-usd">{t("currency.options.usd")}</Label>
        </div>
        <div className="flex items-center space-x-1">
          <RadioGroupItem value="btc" id="currency-btc" />
          <Label htmlFor="currency-btc">{t("currency.options.btc")}</Label>
        </div>
      </RadioGroup>
    </div>
  )
}

interface LayerSelectionProps {
  layer: TrialBalanceLayers
  setLayer: (layer: TrialBalanceLayers) => void
}

export const TrialBalanceLayerSelection: React.FC<LayerSelectionProps> = ({
  layer,
  setLayer,
}) => {
  const t = useTranslations("CurrencyLayerSelection")
  return (
    <div>
      <Label>{t("layer.label")}</Label>
      <RadioGroup
        className="flex items-center py-2.5 bg-muted p-2 rounded-md"
        value={layer}
        onValueChange={(v: TrialBalanceLayers) => setLayer(v)}
      >
        <div className="flex items-center space-x-1 mr-4">
          <RadioGroupItem value="settled" id="layer-settled" />
          <Label htmlFor="layer-settled">{t("layer.options.settled")}</Label>
        </div>
        <div className="flex items-center space-x-1">
          <RadioGroupItem value="pending" id="layer-pending" />
          <Label htmlFor="layer-pending">{t("layer.options.pending")}</Label>
        </div>
      </RadioGroup>
    </div>
  )
}
