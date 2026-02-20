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
import { Input } from "@lana/web/ui/input"
import { Button } from "@lana/web/ui/button"
import { Label } from "@lana/web/ui/label"

import { useWithdrawalInitiateMutation } from "@/lib/graphql/generated"
import { currencyConverter } from "@/lib/utils"
import { useModalNavigation } from "@/hooks/use-modal-navigation"

gql`
  mutation WithdrawalInitiate($input: WithdrawalInitiateInput!) {
    withdrawalInitiate(input: $input) {
      withdrawal {
        ...WithdrawalFields
      }
    }
  }
`

type WithdrawalInitiateDialogProps = {
  setOpenWithdrawalInitiateDialog: (isOpen: boolean) => void
  openWithdrawalInitiateDialog: boolean
  depositAccountId: string
}

export const WithdrawalInitiateDialog: React.FC<WithdrawalInitiateDialogProps> = ({
  setOpenWithdrawalInitiateDialog,
  openWithdrawalInitiateDialog,
  depositAccountId,
}) => {
  const t = useTranslations("Withdrawals.WithdrawalInitiateDialog")

  const handleCloseDialog = () => {
    setOpenWithdrawalInitiateDialog(false)
    setAmount("")
    setReference("")
    setError(null)
    reset()
  }

  const { navigate, isNavigating } = useModalNavigation({
    closeModal: handleCloseDialog,
  })

  const [initiateWithdrawal, { loading, reset }] = useWithdrawalInitiateMutation({
    update: (cache) => {
      cache.modify({
        fields: {
          withdrawals: (_, { DELETE }) => DELETE,
        },
      })
      cache.gc()
    },
  })

  const isLoading = loading || isNavigating
  const [amount, setAmount] = useState<string>("")
  const [reference, setReference] = useState<string>("")
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    try {
      await initiateWithdrawal({
        variables: {
          input: {
            depositAccountId,
            amount: currencyConverter.usdToCents(parseFloat(amount)),
            reference,
          },
        },
        onCompleted: (data) => {
          toast.success(t("success"))
          navigate(`/withdrawals/${data.withdrawalInitiate.withdrawal.publicId}`)
          handleCloseDialog()
        },
      })
    } catch (error) {
      console.error("Error initiating withdrawal:", error)
      setError(error instanceof Error ? error.message : t("errors.unknown"))
    }
  }

  return (
    <Dialog open={openWithdrawalInitiateDialog} onOpenChange={handleCloseDialog}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("title")}</DialogTitle>
          <DialogDescription>{t("description")}</DialogDescription>
        </DialogHeader>
        <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
          <div>
            <Label htmlFor="amount">{t("fields.amount")}</Label>
            <div className="flex items-center gap-1">
              <Input
                data-testid="withdraw-amount-input"
                id="amount"
                type="number"
                required
                placeholder={t("placeholders.amount")}
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
                disabled={isLoading}
                endAdornment={`USD`}
              />
            </div>
          </div>
          <div>
            <Label htmlFor="reference">{t("fields.reference")}</Label>
            <Input
              id="reference"
              type="text"
              placeholder={t("placeholders.reference")}
              value={reference}
              onChange={(e) => setReference(e.target.value)}
              disabled={isLoading}
            />
          </div>
          {error && <p className="text-destructive">{error}</p>}
          <DialogFooter>
            <Button
              type="submit"
              loading={isLoading}
              data-testid="withdraw-submit-button"
            >
              {isLoading ? t("buttons.processing") : t("buttons.submit")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
