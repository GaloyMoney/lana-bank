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
  useDepositAccountUnfreezeMutation,
  GetCustomerBasicDetailsDocument,
} from "@/lib/graphql/generated"

gql`
  mutation DepositAccountUnfreeze($input: DepositAccountUnfreezeInput!) {
    depositAccountUnfreeze(input: $input) {
      account {
        id
      }
    }
  }
`

type UnfreezeDepositAccountDialogProps = {
  setOpenUnfreezeDialog: (isOpen: boolean) => void
  openUnfreezeDialog: boolean
  depositAccountId: string
}

export const UnfreezeDepositAccountDialog: React.FC<UnfreezeDepositAccountDialogProps> = ({
  setOpenUnfreezeDialog,
  openUnfreezeDialog,
  depositAccountId,
}) => {
  const t = useTranslations("Customers.CustomerDetails.unfreezeDepositAccount")
  const commonT = useTranslations("Common")

  const [unfreezeDepositAccount, { loading, reset }] = useDepositAccountUnfreezeMutation({
    refetchQueries: [GetCustomerBasicDetailsDocument],
  })
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    try {
      const result = await unfreezeDepositAccount({
        variables: {
          input: {
            depositAccountId,
          },
        },
      })

      if (result.data?.depositAccountUnfreeze) {
        toast.success(t("success"))
        handleCloseDialog()
      } else {
        setError(commonT("error"))
      }
    } catch (error) {
      console.error("Error unfreezing deposit account:", error)
      setError(error instanceof Error && error.message ? error.message : commonT("error"))
    }
  }

  const handleCloseDialog = () => {
    setOpenUnfreezeDialog(false)
    setError(null)
    reset()
  }

  return (
    <Dialog open={openUnfreezeDialog} onOpenChange={handleCloseDialog}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("title")}</DialogTitle>
          <DialogDescription>{t("description")}</DialogDescription>
        </DialogHeader>
        <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
          {error && <p className="text-destructive">{error}</p>}
          <DialogFooter>
            <Button
              type="submit"
              variant="default"
              disabled={loading}
              data-testid="unfreeze-deposit-account-dialog-button"
            >
              {t("buttons.unfreeze")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

export default UnfreezeDepositAccountDialog
