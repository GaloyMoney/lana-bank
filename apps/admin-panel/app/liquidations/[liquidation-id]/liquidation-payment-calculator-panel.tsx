"use client"

import React, { useState, useEffect } from "react"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"
import { toast } from "sonner"
import { X } from "lucide-react"

import { Button } from "@lana/web/ui/button"
import { Input } from "@lana/web/ui/input"
import { Label } from "@lana/web/ui/label"
import { Card, CardContent, CardHeader, CardTitle } from "@lana/web/ui/card"

import {
  useLiquidationPaymentCalculateLazyQuery,
  CvlPct,
  LiquidationPaymentCalculateInput,
  FiniteCvlPct,
} from "@/lib/graphql/generated"
import { currencyConverter } from "@/lib/utils"

gql`
  query LiquidationPaymentCalculate($input: LiquidationPaymentCalculateInput!) {
    liquidationPaymentCalculate(input: $input) {
      toReceive
      toLiquidate
      targetCvl {
        __typename
        ... on FiniteCvlPct {
          value
        }
        ... on InfiniteCvlPct {
          isInfinite
        }
      }
    }
  }
`

type LiquidationPaymentCalculatorPanelProps = {
  open: boolean
  onClose: () => void
  liquidationId: string
  outstanding: number
  defaultToReceive?: number
  defaultToLiquidate?: number
}

export const LiquidationPaymentCalculatorPanel: React.FC<
  LiquidationPaymentCalculatorPanelProps
> = ({
  open,
  onClose,
  liquidationId,
  outstanding,
  defaultToReceive,
  defaultToLiquidate,
}) => {
  const t = useTranslations("Liquidations.LiquidationDetails.calculator")
  const commonT = useTranslations("Common")

  const [calculatePayment, { loading }] =
    useLiquidationPaymentCalculateLazyQuery()

  const [toReceive, setToReceive] = useState(
    defaultToReceive ? (defaultToReceive / 100).toString() : ""
  )
  const [toLiquidate, setToLiquidate] = useState(
    defaultToLiquidate
      ? (defaultToLiquidate / 100000000).toString()
      : ""
  )
  const [targetCvl, setTargetCvl] = useState("")

  const [results, setResults] = useState<{
    toReceive: number
    toLiquidate: number
    targetCvl: CvlPct
  } | null>(null)

  const [error, setError] = useState<string | null>(null)

  // Reset form when opened
  useEffect(() => {
    if (open) {
      setToReceive(defaultToReceive ? (defaultToReceive / 100).toString() : "")
      setToLiquidate(
        defaultToLiquidate ? (defaultToLiquidate / 100000000).toString() : ""
      )
      setTargetCvl("")
      setResults(null)
      setError(null)
    }
  }, [open, defaultToReceive, defaultToLiquidate])

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    const filledFields = [
      toReceive !== "",
      toLiquidate !== "",
      targetCvl !== "",
    ].filter(Boolean).length

    if (filledFields !== 2) {
      setError(t("validation.twoFieldsRequired"))
      return
    }

    const input: LiquidationPaymentCalculateInput = {
      liquidationId,
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      outstanding: outstanding as any,
    }

    if (toReceive !== "") {
      const toReceiveNum = parseFloat(toReceive)
      if (isNaN(toReceiveNum) || toReceiveNum <= 0) {
        setError(t("validation.invalidAmount"))
        return
      }
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      input.toReceive = Math.round(toReceiveNum * 100) as any
    }

    if (toLiquidate !== "") {
      const toLiquidateNum = parseFloat(toLiquidate)
      if (isNaN(toLiquidateNum) || toLiquidateNum <= 0) {
        setError(t("validation.invalidAmount"))
        return
      }
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      input.toLiquidate = currencyConverter.btcToSatoshi(toLiquidateNum) as any
    }

    if (targetCvl !== "") {
      const targetCvlNum = parseFloat(targetCvl)
      if (isNaN(targetCvlNum) || targetCvlNum < 0) {
        setError(t("validation.invalidCvl"))
        return
      }
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      input.targetCvl = targetCvlNum as any
    }

    try {
      const result = await calculatePayment({
        variables: {
          input,
        },
      })

      if (result.data) {
        setResults({
          toReceive: result.data.liquidationPaymentCalculate.toReceive,
          toLiquidate: result.data.liquidationPaymentCalculate.toLiquidate,
          targetCvl: result.data.liquidationPaymentCalculate.targetCvl,
        })
        toast.success(t("success"))
      }
    } catch (err) {
      console.error("Error calculating payment:", err)
      setError(
        err instanceof Error
          ? t("errors.calculationFailed", { error: err.message })
          : commonT("error")
      )
    }
  }

  const handleReset = () => {
    setToReceive(defaultToReceive ? (defaultToReceive / 100).toString() : "")
    setToLiquidate(
      defaultToLiquidate ? (defaultToLiquidate / 100000000).toString() : ""
    )
    setTargetCvl("")
    setResults(null)
    setError(null)
  }

  const formatCvl = (cvl: CvlPct): string => {
    if (cvl.__typename === "InfiniteCvlPct") {
      return t("results.infinite")
    }
    if (cvl.__typename === "FiniteCvlPct") {
      return `${(cvl as FiniteCvlPct).value}%`
    }
    return t("results.infinite")
  }

  if (!open) return null

  return (
    <Card className="animate-in slide-in-from-top-2 duration-200">
      <CardHeader className="pb-4">
        <div className="flex items-center justify-between">
          <CardTitle className="text-lg">
            {t("title")}
          </CardTitle>
          <Button
            type="button"
            variant="ghost"
            size="icon"
            onClick={onClose}
            className="h-8 w-8"
          >
            <X className="h-4 w-4" />
          </Button>
        </div>
        <p className="text-sm text-muted-foreground">{t("description")}</p>
      </CardHeader>
      <CardContent>
        <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="flex flex-col gap-2">
              <Label htmlFor="toReceive">{t("fields.toReceive")}</Label>
              <Input
                id="toReceive"
                type="number"
                value={toReceive}
                onChange={(e) => setToReceive(e.target.value)}
                placeholder={t("fields.toReceivePlaceholder")}
                disabled={loading}
                endAdornment="USD"
                step="0.01"
                min="0"
              />
            </div>
            <div className="flex flex-col gap-2">
              <Label htmlFor="toLiquidate">{t("fields.toLiquidate")}</Label>
              <Input
                id="toLiquidate"
                type="number"
                value={toLiquidate}
                onChange={(e) => setToLiquidate(e.target.value)}
                placeholder={t("fields.toLiquidatePlaceholder")}
                disabled={loading}
                endAdornment="BTC"
                step="0.00000001"
                min="0"
              />
            </div>
            <div className="flex flex-col gap-2">
              <Label htmlFor="targetCvl">{t("fields.targetCvl")}</Label>
              <Input
                id="targetCvl"
                type="number"
                value={targetCvl}
                onChange={(e) => setTargetCvl(e.target.value)}
                placeholder={t("fields.targetCvlPlaceholder")}
                disabled={loading}
                endAdornment="%"
                step="0.01"
                min="0"
              />
            </div>
          </div>

          {error && <p className="text-destructive text-sm">{error}</p>}

          {results && (
            <div className={`bg-muted rounded-md p-4 space-y-3 transition-opacity duration-200 ${loading ? 'opacity-50' : 'opacity-100'}`}>
              <h4 className="font-semibold text-sm">{t("results.title")}</h4>
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm">
                <div className="flex flex-col gap-1">
                  <span className="text-muted-foreground">
                    {t("results.toReceive")}
                  </span>
                  <span className="font-medium">
                    ${(results.toReceive / 100).toFixed(2)} USD
                  </span>
                </div>
                <div className="flex flex-col gap-1">
                  <span className="text-muted-foreground">
                    {t("results.toLiquidate")}
                  </span>
                  <span className="font-medium">
                    {(results.toLiquidate / 100000000).toFixed(8)} BTC
                  </span>
                </div>
                <div className="flex flex-col gap-1">
                  <span className="text-muted-foreground">
                    {t("results.targetCvl")}
                  </span>
                  <span className="font-medium">{formatCvl(results.targetCvl)}</span>
                </div>
              </div>
            </div>
          )}

          <div className="flex flex-wrap gap-2 justify-end pt-2">
            <Button
              type="button"
              variant="outline"
              onClick={handleReset}
              disabled={loading}
            >
              {commonT("reset")}
            </Button>
            <Button
              type="submit"
              loading={loading}
              data-testid="liquidation-payment-calculator-submit-button"
            >
              {t("buttons.calculate")}
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  )
}

export default LiquidationPaymentCalculatorPanel
