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
  useDepositAccountFreezeMutation,
  GetDepositAccountDetailsQuery,
  GetDepositAccountDetailsDocument,
} from "@/lib/graphql/generated"
import { DetailItem, DetailsGroup } from "@/components/details"
import Balance from "@/components/balance/balance"

gql`
  mutation DepositAccountFreeze($input: DepositAccountFreezeInput!) {
    depositAccountFreeze(input: $input) {
      account {
        id
      }
    }
  }
`

type FreezeDepositAccountDialogProps = {
  setOpenFreezeDialog: (isOpen: boolean) => void
  openFreezeDialog: boolean
  depositAccountId: string
  balance: NonNullable<
    GetDepositAccountDetailsQuery["depositAccountByPublicId"]
  >["balance"]
}

export const FreezeDepositAccountDialog: React.FC<FreezeDepositAccountDialogProps> = ({
  setOpenFreezeDialog,
  openFreezeDialog,
  depositAccountId,
  balance,
}) => {
  const t = useTranslations("DepositAccounts.DepositAccountDetails.freezeDepositAccount")

  const [freezeDepositAccount, { loading, reset }] = useDepositAccountFreezeMutation({
    refetchQueries: [GetDepositAccountDetailsDocument],
  })
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    try {
      const result = await freezeDepositAccount({
        variables: {
          input: {
            depositAccountId,
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
      console.error("Error freezing deposit account:", error)
      setError(error instanceof Error ? error.message : t("errors.unknown"))
    }
  }

  const handleCloseDialog = () => {
    setOpenFreezeDialog(false)
    setError(null)
    reset()
  }

  return (
    <Dialog open={openFreezeDialog} onOpenChange={handleCloseDialog}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("title")}</DialogTitle>
          <DialogDescription>{t("description")}</DialogDescription>
        </DialogHeader>
        <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
          <DetailsGroup layout="horizontal">
            <DetailItem
              label={t("fields.settledBalance")}
              value={<Balance amount={balance.settled} currency="usd" />}
            />
            <DetailItem
              label={t("fields.pendingBalance")}
              value={<Balance amount={balance.pending} currency="usd" />}
            />
          </DetailsGroup>
          {error && <p className="text-destructive">{error}</p>}
          <DialogFooter>
            <Button
              type="submit"
              variant="destructive"
              disabled={loading}
              data-testid="freeze-deposit-account-dialog-button"
            >
              {t("buttons.freeze")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

export default FreezeDepositAccountDialog
