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

import { useDepositAccountCloseMutation } from "@/lib/graphql/generated"

gql`
  mutation DepositAccountClose($input: DepositAccountCloseInput!) {
    depositAccountClose(input: $input) {
      account {
        ...DepositAccountDetailsFragment
      }
    }
  }
`

type CloseDepositAccountDialogProps = {
  setOpenCloseDialog: (isOpen: boolean) => void
  openCloseDialog: boolean
  depositAccountId: string
}

export const CloseDepositAccountDialog: React.FC<CloseDepositAccountDialogProps> = ({
  setOpenCloseDialog,
  openCloseDialog,
  depositAccountId,
}) => {
  const t = useTranslations("DepositAccounts.DepositAccountDetails.closeDepositAccount")
  const commonT = useTranslations("Common")

  const [closeDepositAccount, { loading, reset }] = useDepositAccountCloseMutation()
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    try {
      const result = await closeDepositAccount({
        variables: {
          input: {
            depositAccountId,
          },
        },
      })

      if (result.data) {
        toast.success(t("success"))
        handleCloseDialog()
      }
    } catch (error) {
      console.error("Error closing deposit account:", error)
      setError(error instanceof Error ? error.message : commonT("error"))
    }
  }

  const handleCloseDialog = () => {
    setOpenCloseDialog(false)
    setError(null)
    reset()
  }

  return (
    <Dialog open={openCloseDialog} onOpenChange={handleCloseDialog}>
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
              variant="destructive"
              disabled={loading}
              data-testid="close-deposit-account-dialog-button"
            >
              {t("buttons.close")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

export default CloseDepositAccountDialog
