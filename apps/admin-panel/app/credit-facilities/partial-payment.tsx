import React, { useState } from "react"
import { gql } from "@apollo/client"
import { toast } from "sonner"
import { useTranslations } from "next-intl"

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
  useCreditFacilityPartialPaymentRecordMutation,
  useCreditFacilityPartialPaymentWithDateRecordMutation,
} from "@/lib/graphql/generated"
import { UsdCents } from "@/types"
import { getCurrentLocalDate } from "@/lib/utils"

gql`
  mutation CreditFacilityPartialPaymentRecord(
    $input: CreditFacilityPartialPaymentRecordInput!
  ) {
    creditFacilityPartialPaymentRecord(input: $input) {
      creditFacility {
        id
        creditFacilityId
        ...CreditFacilityHistoryFragment

        ...CreditFacilityLayoutFragment
      }
    }
  }
`

gql`
  mutation CreditFacilityPartialPaymentWithDateRecord(
    $input: CreditFacilityPartialPaymentWithDateRecordInput!
  ) {
    creditFacilityPartialPaymentWithDateRecord(input: $input) {
      creditFacility {
        id
        creditFacilityId
        ...CreditFacilityHistoryFragment
        ...CreditFacilityLayoutFragment
      }
    }
  }
`

type CreditFacilityPartialPaymentDialogProps = {
  setOpenDialog: (isOpen: boolean) => void
  openDialog: boolean
  creditFacilityId: string
  userCanRecordPaymentWithDate: boolean
}

export const CreditFacilityPartialPaymentDialog: React.FC<
  CreditFacilityPartialPaymentDialogProps
> = ({ setOpenDialog, openDialog, creditFacilityId, userCanRecordPaymentWithDate }) => {
  const t = useTranslations(
    "CreditFacilities.CreditFacilityDetails.CreditFacilityPartialPayment",
  )

  const [partialPaymentCreditFacility, { loading, reset }] =
    useCreditFacilityPartialPaymentRecordMutation()
  const [
    partialPaymentWithDateCreditFacility,
    { loading: loadingWithDate, reset: resetWithDate },
  ] = useCreditFacilityPartialPaymentWithDateRecordMutation()
  const [error, setError] = useState<string | null>(null)
  const [amount, setAmount] = useState<string>("")
  const [effectiveDate, setEffectiveDate] = useState<string>(getCurrentLocalDate())

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    const amountInCents = Math.round(parseFloat(amount) * 100)

    if (isNaN(amountInCents) || amountInCents <= 0) {
      setError(t("form.errors.invalidAmount"))
      return
    }

    try {
      if (userCanRecordPaymentWithDate) {
        await partialPaymentWithDateCreditFacility({
          variables: {
            input: {
              creditFacilityId,
              amount: amountInCents as UsdCents,
              effective: effectiveDate,
            },
          },
          onCompleted: (data) => {
            if (data.creditFacilityPartialPaymentWithDateRecord) {
              toast.success(t("messages.success"))
              handleCloseDialog()
            }
          },
        })
      } else {
        await partialPaymentCreditFacility({
          variables: {
            input: {
              creditFacilityId,
              amount: amountInCents as UsdCents,
            },
          },
          onCompleted: (data) => {
            if (data.creditFacilityPartialPaymentRecord) {
              toast.success(t("messages.success"))
              handleCloseDialog()
            }
          },
        })
      }
    } catch (error) {
      console.error("Error processing partial payment:", error)
      if (error instanceof Error) {
        setError(error.message)
      } else {
        setError(t("form.errors.unknownError"))
      }
    }
  }

  const handleCloseDialog = () => {
    setOpenDialog(false)
    setError(null)
    setAmount("")
    setEffectiveDate(getCurrentLocalDate())
    reset()
    resetWithDate()
  }

  return (
    <Dialog open={openDialog} onOpenChange={handleCloseDialog}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("dialog.title")}</DialogTitle>
          <DialogDescription>{t("dialog.description")}</DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit} className="flex flex-col gap-4">
          <div>
            <Label>{t("form.labels.amount")}</Label>
            <div className="flex items-center gap-1">
              <Input
                data-testid="facility-partial-payment-amount-input"
                type="number"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
                placeholder={t("form.placeholders.amount")}
                min={0}
                endAdornment={`USD`}
              />
            </div>
          </div>

          <div>
            <Label htmlFor="effectiveDate">{t("form.labels.effectiveDate")}</Label>
            <Input
              id="effectiveDate"
              data-testid="facility-partial-payment-date-input"
              type="date"
              value={effectiveDate}
              onChange={(e) => setEffectiveDate(e.target.value)}
              disabled={!userCanRecordPaymentWithDate}
              required
            />
          </div>

          {error && <p className="text-destructive text-sm">{error}</p>}

          <DialogFooter>
            <Button type="button" variant="ghost" onClick={handleCloseDialog}>
              {t("form.buttons.cancel")}
            </Button>
            <Button
              type="submit"
              loading={loading || loadingWithDate}
              data-testid="facility-partial-payment-submit-button"
            >
              {t("form.buttons.processPayment")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
