"use client"

import React, { useState } from "react"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"
import { toast } from "sonner"
import { Calculator } from "lucide-react"

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@lana/web/ui/dialog"
import { Button } from "@lana/web/ui/button"
import { Input } from "@lana/web/ui/input"
import { Label } from "@lana/web/ui/label"

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

type LiquidationPaymentCalculatorDialogProps = {
  open: boolean
  onOpenChange: (isOpen: boolean) => void
  liquidationId: string
  outstanding: number
  defaultToReceive?: number
  defaultToLiquidate?: number
}

export const LiquidationPaymentCalculatorDialog: React.FC<
  LiquidationPaymentCalculatorDialogProps
> = ({
  open,
  onOpenChange,
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

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    setResults(null)

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

  const handleCloseDialog = () => {
    setToReceive(defaultToReceive ? (defaultToReceive / 100).toString() : "")
    setToLiquidate(
      defaultToLiquidate ? (defaultToLiquidate / 100000000).toString() : ""
    )
    setTargetCvl("")
    setResults(null)
    setError(null)
    onOpenChange(false)
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

  return (
    <Dialog open={open} onOpenChange={handleCloseDialog}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Calculator className="h-5 w-5" />
            {t("title")}
          </DialogTitle>
          <DialogDescription>{t("description")}</DialogDescription>
        </DialogHeader>
        <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
          <div className="grid grid-cols-1 gap-4">
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
            <div className="bg-muted rounded-md p-4 space-y-3">
              <h4 className="font-semibold text-sm">{t("results.title")}</h4>
              <div className="grid grid-cols-1 gap-2 text-sm">
                <div className="flex justify-between">
                  <span className="text-muted-foreground">
                    {t("results.toReceive")}
                  </span>
                  <span className="font-medium">
                    ${(results.toReceive / 100).toFixed(2)} USD
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">
                    {t("results.toLiquidate")}
                  </span>
                  <span className="font-medium">
                    {(results.toLiquidate / 100000000).toFixed(8)} BTC
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">
                    {t("results.targetCvl")}
                  </span>
                  <span className="font-medium">{formatCvl(results.targetCvl)}</span>
                </div>
              </div>
            </div>
          )}

          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={handleReset}
              disabled={loading}
            >
              {commonT("reset")}
            </Button>
            <Button
              type="button"
              variant="outline"
              onClick={handleCloseDialog}
              disabled={loading}
            >
              {commonT("cancel")}
            </Button>
            <Button
              type="submit"
              loading={loading}
              data-testid="liquidation-payment-calculator-dialog-button"
            >
              {t("buttons.calculate")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

export default LiquidationPaymentCalculatorDialog
