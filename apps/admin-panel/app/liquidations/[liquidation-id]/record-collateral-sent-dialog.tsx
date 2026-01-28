"use client"

import React, { useState } from "react"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"
import { toast } from "sonner"

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

import { useLiquidationRecordCollateralSentMutation } from "@/lib/graphql/generated"
import { currencyConverter } from "@/lib/utils"

gql`
  mutation LiquidationRecordCollateralSent(
    $input: LiquidationRecordCollateralSentInput!
  ) {
    liquidationRecordCollateralSent(input: $input) {
      collateral {
        collateralId
      }
    }
  }
`

type RecordCollateralSentDialogProps = {
  open: boolean
  onOpenChange: (isOpen: boolean) => void
  collateralId: string
}

export const RecordCollateralSentDialog: React.FC<RecordCollateralSentDialogProps> = ({
  open,
  onOpenChange,
  collateralId,
}) => {
  const t = useTranslations("Liquidations.LiquidationDetails.recordCollateralSent")
  const commonT = useTranslations("Common")

  const [recordCollateralSent, { loading, reset }] =
    useLiquidationRecordCollateralSentMutation()
  const [amount, setAmount] = useState("")
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    if (!amount || parseFloat(amount) <= 0) {
      setError(t("errors.invalidAmount"))
      return
    }

    try {
      const result = await recordCollateralSent({
        variables: {
          input: {
            collateralId,
            amount: currencyConverter.btcToSatoshi(Number(amount)),
          },
        },
      })

      if (result.data) {
        toast.success(t("success"))
        handleCloseDialog()
      }
    } catch (error) {
      console.error("Error recording collateral sent:", error)
      setError(error instanceof Error ? error.message : commonT("error"))
    }
  }

  const handleCloseDialog = () => {
    setAmount("")
    setError(null)
    reset()
    onOpenChange(false)
  }

  return (
    <Dialog open={open} onOpenChange={handleCloseDialog}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("title")}</DialogTitle>
          <DialogDescription>{t("description")}</DialogDescription>
        </DialogHeader>
        <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
          <div className="flex flex-col gap-2">
            <Label htmlFor="amount">{t("fields.amount")}</Label>
            <Input
              id="amount"
              type="number"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              placeholder={t("fields.amountPlaceholder")}
              disabled={loading}
              endAdornment="BTC"
              step="0.00000001"
              min="0"
              required
            />
          </div>
          {error && <p className="text-destructive text-sm">{error}</p>}
          <DialogFooter>
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
              data-testid="record-collateral-sent-dialog-button"
            >
              {t("buttons.submit")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

export default RecordCollateralSentDialog
