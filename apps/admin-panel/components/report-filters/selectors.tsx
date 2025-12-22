"use client"
import React from "react"
import { useTranslations } from "next-intl"
import { Label } from "@lana/web/ui/label"
import { Tabs, TabsList, TabsTrigger } from "@lana/web/ui/tabs"

import { Currency } from "@/components/balance/balance"

export type ReportLayer = "settled" | "pending"

type CurrencySelectionProps = {
  currency: Currency
  setCurrency: (currency: Currency) => void
}

export const CurrencySelection: React.FC<CurrencySelectionProps> = ({
  currency,
  setCurrency,
}) => {
  const t = useTranslations("CurrencyLayerSelection")
  return (
    <div>
      <Label>{t("currency.label")}</Label>
      <Tabs value={currency} onValueChange={(v) => setCurrency(v as Currency)}>
        <TabsList className="border rounded-md ">
          <TabsTrigger value="usd">{t("currency.options.usd")}</TabsTrigger>
          <TabsTrigger value="btc">{t("currency.options.btc")}</TabsTrigger>
        </TabsList>
      </Tabs>
    </div>
  )
}

type LayerSelectionProps = {
  layer: ReportLayer
  setLayer: (layer: ReportLayer) => void
}

export const LayerSelection: React.FC<LayerSelectionProps> = ({ layer, setLayer }) => {
  const t = useTranslations("CurrencyLayerSelection")
  return (
    <div>
      <Label>{t("layer.label")}</Label>
      <Tabs value={layer} onValueChange={(v) => setLayer(v as ReportLayer)}>
        <TabsList className="border rounded-md ">
          <TabsTrigger value="settled">{t("layer.options.settled")}</TabsTrigger>
          <TabsTrigger value="pending">{t("layer.options.pending")}</TabsTrigger>
        </TabsList>
      </Tabs>
    </div>
  )
}
