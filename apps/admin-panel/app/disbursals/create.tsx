import React, { useState } from "react"
import { gql } from "@apollo/client"
import { toast } from "sonner"
import { useRouter } from "next/navigation"
import { useTranslations } from "next-intl"

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@lana/web/ui/dialog"
import { Input } from "@lana/web/ui/input"
import { Button } from "@lana/web/ui/button"
import { Label } from "@lana/web/ui/label"

import { useCreditFacilityDisbursalInitiateMutation } from "@/lib/graphql/generated"
import { currencyConverter } from "@/lib/utils"

gql`
  mutation CreditFacilityDisbursalInitiate(
    $input: CreditFacilityDisbursalInitiateInput!
  ) {
    creditFacilityDisbursalInitiate(input: $input) {
      disbursal {
        id
        disbursalId
        publicId
        amount
        status
        createdAt
        creditFacility {
          id
          disbursals {
            ...DisbursalOnFacilityPage
          }
          ...CreditFacilityHistoryFragment
          ...CreditFacilityLayoutFragment
        }
      }
    }
  }
`

type CreditFacilityDisbursalInitiateDialogProps = {
  setOpenDialog: (isOpen: boolean) => void
  openDialog: boolean
  creditFacilityId: string
}

export const CreditFacilityDisbursalInitiateDialog: React.FC<
  CreditFacilityDisbursalInitiateDialogProps
> = ({ setOpenDialog, openDialog, creditFacilityId }) => {
  const t = useTranslations("Disbursals.DisbursalDetails.CreditFacilityDisbursalInitiate")
  const router = useRouter()
  const [initiateDisbursal, { loading, reset }] =
    useCreditFacilityDisbursalInitiateMutation({
      update: (cache) => {
        cache.modify({
          fields: {
            disbursals: (_, { DELETE }) => DELETE,
          },
        })
        cache.gc()
      },
    })
  const [amount, setAmount] = useState<string>("")
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    try {
      await initiateDisbursal({
        variables: {
          input: {
            creditFacilityId,
            amount: currencyConverter.usdToCents(parseFloat(amount)),
          },
        },
        onCompleted: (data) => {
          if (data.creditFacilityDisbursalInitiate) {
            router.push(
              `/disbursals/${data.creditFacilityDisbursalInitiate.disbursal.publicId}`,
            )
            toast.success(t("messages.success"))
            handleCloseDialog()
          }
        },
      })
    } catch (error) {
      console.error("Error initiating disbursal:", error)
      if (error instanceof Error) {
        setError(error.message)
      } else {
        setError(t("messages.unknownError"))
      }
    }
  }

  const handleCloseDialog = () => {
    setOpenDialog(false)
    setAmount("")
    setError(null)
    reset()
  }

  return (
    <Dialog open={openDialog} onOpenChange={handleCloseDialog}>
      <DialogContent data-testid="disbursal-dialog-content">
        <DialogHeader>
          <DialogTitle>{t("dialog.title")}</DialogTitle>
          <DialogDescription>{t("dialog.description")}</DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit}>
          <div className="mb-4">
            <Label htmlFor="amount">{t("form.labels.amount")}</Label>
            <div className="flex items-center gap-1">
              <Input
                id="amount"
                type="number"
                required
                placeholder={t("form.placeholders.amount")}
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
                data-testid="disbursal-amount-input"
                endAdornment={`USD`}
              />
            </div>
          </div>
          {error && <p className="text-destructive mb-4">{error}</p>}
          <DialogFooter>
            <Button type="button" variant="ghost" onClick={handleCloseDialog}>
              {t("buttons.cancel")}
            </Button>
            <Button type="submit" loading={loading} data-testid="disbursal-submit-button">
              {t("buttons.submit")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
