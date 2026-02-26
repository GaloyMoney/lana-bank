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
  useWithdrawalRevertMutation,
} from "@/lib/graphql/generated"
import Balance from "@/components/balance/balance"
import { DetailItem, DetailsGroup } from "@/components/details"
import { UsdCents } from "@/types"

gql`
  mutation WithdrawalRevert($input: WithdrawalRevertInput!) {
    withdrawalRevert(input: $input) {
      withdrawal {
        ...WithdrawDetailsPageFragment
      }
    }
  }
`

type WithdrawalRevertDialogProps = {
  setOpenWithdrawalRevertDialog: (isOpen: boolean) => void
  openWithdrawalRevertDialog: boolean
  withdrawalData: NonNullable<GetWithdrawalDetailsQuery["withdrawalByPublicId"]>
}

export const WithdrawalRevertDialog: React.FC<WithdrawalRevertDialogProps> = ({
  setOpenWithdrawalRevertDialog,
  openWithdrawalRevertDialog,
  withdrawalData,
}) => {
  const t = useTranslations("Withdrawals.WithdrawDetails.WithdrawalRevertDialog")
  const [revertWithdrawal, { loading, reset }] = useWithdrawalRevertMutation()
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    try {
      const result = await revertWithdrawal({
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
      console.error("Error reverting withdrawal:", error)
      setError(error instanceof Error ? error.message : t("errors.unknown"))
    }
  }

  const handleCloseDialog = () => {
    setOpenWithdrawalRevertDialog(false)
    setError(null)
    reset()
  }

  return (
    <Dialog open={openWithdrawalRevertDialog} onOpenChange={handleCloseDialog}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("title")}</DialogTitle>
          <DialogDescription>{t("description")}</DialogDescription>
        </DialogHeader>
        <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
          <DetailsGroup layout="horizontal">
            <DetailItem
              label={t("fields.customerEmail")}
              value={withdrawalData.account.customerId}
            />
            <DetailItem
              label={t("fields.amount")}
              value={
                <Balance amount={withdrawalData.amount as UsdCents} currency="usd" />
              }
            />
          </DetailsGroup>
          {error && <p className="text-destructive">{error}</p>}
          <DialogFooter>
            <Button
              type="submit"
              loading={loading}
              data-testid="withdrawal-revert-dialog-button"
            >
              {t("buttons.revert")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
