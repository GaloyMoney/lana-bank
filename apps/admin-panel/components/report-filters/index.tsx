import React from "react"
import { Label } from "@lana/web/ui/label"
import { Separator } from "@lana/web/ui/separator"
import { useTranslations } from "next-intl"

import { CurrencySelection, LayerSelection, ReportLayer } from "./selectors"

import { DateRangeSelector } from "@/components/date-range-picker"

import { Currency } from "@/components/balance/balance"

type ReportFiltersProps = {
  dateRange: { from: string; until: string }
  onDateChange: (dateRange: { from: string; until: string }) => void
  currency?: Currency
  onCurrencyChange?: (currency: Currency) => void
  layer?: ReportLayer
  onLayerChange?: (layer: ReportLayer) => void
}

export const ReportFilters: React.FC<ReportFiltersProps> = ({
  dateRange,
  onDateChange,
  currency,
  onCurrencyChange,
  layer,
  onLayerChange,
}) => {
  const t = useTranslations("DateRangePicker")
  return (
    <div className="flex items-center rounded-md flex-wrap w-fit gap-2 mb-2">
      <div>
        <Label>{t("dateRange")}</Label>
        <DateRangeSelector initialDateRange={dateRange} onDateChange={onDateChange} />
      </div>
      {currency && onCurrencyChange && (
        <>
          <div className="flex items-center h-14">
            <Separator orientation="vertical" />
          </div>
          <div className="flex flex-col gap-2">
            <CurrencySelection currency={currency} setCurrency={onCurrencyChange} />
          </div>
        </>
      )}
      {layer && onLayerChange && (
        <>
          <div className="flex items-center h-14">
            <Separator orientation="vertical" />
          </div>
          <div className="flex flex-col gap-2">
            <LayerSelection layer={layer} setLayer={onLayerChange} />
          </div>
        </>
      )}
    </div>
  )
}
