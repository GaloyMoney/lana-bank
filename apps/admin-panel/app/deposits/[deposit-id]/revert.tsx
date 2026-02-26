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

import { GetDepositDetailsQuery, useDepositRevertMutation } from "@/lib/graphql/generated"
import Balance from "@/components/balance/balance"
import { DetailItem, DetailsGroup } from "@/components/details"
import { UsdCents } from "@/types"

gql`
  mutation DepositRevert($input: DepositRevertInput!) {
    depositRevert(input: $input) {
      deposit {
        ...DepositDetailsPageFragment
      }
    }
  }
`

type DepositRevertDialogProps = {
  setOpenDepositRevertDialog: (isOpen: boolean) => void
  openDepositRevertDialog: boolean
  depositData: NonNullable<GetDepositDetailsQuery["depositByPublicId"]>
}

export const DepositRevertDialog: React.FC<DepositRevertDialogProps> = ({
  setOpenDepositRevertDialog,
  openDepositRevertDialog,
  depositData,
}) => {
  const t = useTranslations("Deposits.DepositDetails.DepositRevertDialog")
  const [revertDeposit, { loading, reset }] = useDepositRevertMutation()
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    try {
      const result = await revertDeposit({
        variables: {
          input: {
            depositId: depositData.depositId,
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
      console.error("Error reverting deposit:", error)
      setError(error instanceof Error ? error.message : t("errors.unknown"))
    }
  }

  const handleCloseDialog = () => {
    setOpenDepositRevertDialog(false)
    setError(null)
    reset()
  }

  return (
    <Dialog open={openDepositRevertDialog} onOpenChange={handleCloseDialog}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("title")}</DialogTitle>
          <DialogDescription>{t("description")}</DialogDescription>
        </DialogHeader>
        <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
          <DetailsGroup layout="horizontal">
            <DetailItem
              label={t("fields.accountId")}
              value={depositData.accountId}
            />
            <DetailItem
              label={t("fields.amount")}
              value={<Balance amount={depositData.amount as UsdCents} currency="usd" />}
            />
          </DetailsGroup>
          {error && <p className="text-destructive">{error}</p>}
          <DialogFooter>
            <Button
              type="submit"
              loading={loading}
              data-testid="deposit-revert-dialog-button"
            >
              {t("buttons.revert")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
