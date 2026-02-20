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

import {
  GetWithdrawalDetailsQuery,
  useWithdrawalCancelMutation,
} from "@/lib/graphql/generated"
import Balance from "@/components/balance/balance"
import { DetailItem, DetailsGroup } from "@/components/details"
import { UsdCents } from "@/types"

gql`
  mutation WithdrawalCancel($input: WithdrawalCancelInput!) {
    withdrawalCancel(input: $input) {
      withdrawal {
        ...WithdrawDetailsPageFragment
      }
    }
  }
`

type WithdrawalCancelDialogProps = {
  setOpenWithdrawalCancelDialog: (isOpen: boolean) => void
  openWithdrawalCancelDialog: boolean
  withdrawalData: NonNullable<GetWithdrawalDetailsQuery["withdrawalByPublicId"]>
}

export const WithdrawalCancelDialog: React.FC<WithdrawalCancelDialogProps> = ({
  setOpenWithdrawalCancelDialog,
  openWithdrawalCancelDialog,
  withdrawalData,
}) => {
  const t = useTranslations("Withdrawals.WithdrawDetails.WithdrawalCancelDialog")
  const [cancelWithdrawal, { loading, reset }] = useWithdrawalCancelMutation()
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    try {
      const result = await cancelWithdrawal({
        variables: {
          input: {
            withdrawalId: withdrawalData.withdrawalId,
          },
        },
      })
      if (result.data) {
        toast.success(t("success"))
        handleCloseDialog()
      } else {
        throw new Error(t("errors.noData"))
      }
    } catch (error) {
      console.error("Error canceling withdrawal:", error)
      setError(error instanceof Error ? error.message : t("errors.unknown"))
    }
  }

  const handleCloseDialog = () => {
    setOpenWithdrawalCancelDialog(false)
    setError(null)
    reset()
  }

  return (
    <Dialog open={openWithdrawalCancelDialog} onOpenChange={handleCloseDialog}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("title")}</DialogTitle>
          <DialogDescription>{t("description")}</DialogDescription>
        </DialogHeader>
        <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
          <DetailsGroup layout="horizontal">
            <DetailItem
              label={t("fields.accountId")}
              value={withdrawalData.accountId}
            />
            <DetailItem
              label={t("fields.amount")}
              value={
                <Balance amount={withdrawalData.amount as UsdCents} currency="usd" />
              }
            />
            <DetailItem
              label={t("fields.withdrawalReference")}
              value={
                withdrawalData.reference === withdrawalData.withdrawalId
                  ? t("values.na")
                  : withdrawalData.reference
              }
            />
          </DetailsGroup>
          {error && <p className="text-destructive">{error}</p>}
          <DialogFooter>
            <Button
              type="submit"
              disabled={loading}
              data-testid="withdrawal-cancel-dialog-button"
            >
              {loading ? t("buttons.canceling") : t("buttons.confirm")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
