import React from "react"

import { Currency } from "../balance/balance"
import { Label } from "../primitive/label"
import { RadioGroup, RadioGroupItem } from "../primitive/radio-group"

interface CurrencyLayerSelectionProps {
  currency: Currency
  setCurrency: (currency: Currency) => void
  layer: Layers
  setLayer: (layer: Layers) => void
}

export const CurrencyLayerSelection: React.FC<CurrencyLayerSelectionProps> = ({
  currency,
  setCurrency,
  layer,
  setLayer,
}) => {
  return (
    <div>
      <div className="flex items-center py-3 mt-2">
        <div className="w-28">Currency:</div>
        <RadioGroup
          className="flex items-center space-x-4"
          defaultValue="btc"
          value={currency}
          onValueChange={(v: Currency) => setCurrency(v)}
        >
          <div className="flex items-center space-x-2">
            <RadioGroupItem value="usd" id="currency-usd" />
            <Label htmlFor="currency-usd">USD</Label>
          </div>
          <div className="flex items-center space-x-2">
            <RadioGroupItem value="btc" id="currency-btc" />
            <Label htmlFor="currency-btc">BTC</Label>
          </div>
        </RadioGroup>
      </div>
      <div className="flex items-center py-3">
        <div className="w-28">Layer:</div>
        <RadioGroup
          className="flex items-center space-x-4"
          defaultValue="all"
          value={layer}
          onValueChange={(v: Layers) => setLayer(v)}
        >
          <div className="flex items-center space-x-2">
            <RadioGroupItem value="all" id="layer-all" />
            <Label htmlFor="layer-all">All</Label>
          </div>
          <div className="flex items-center space-x-2">
            <RadioGroupItem value="settled" id="layer-settled" />
            <Label htmlFor="layer-settled">Settled</Label>
          </div>
          <div className="flex items-center space-x-2">
            <RadioGroupItem value="pending" id="layer-pending" />
            <Label htmlFor="layer-pending">Pending</Label>
          </div>
        </RadioGroup>
      </div>
    </div>
  )
}
