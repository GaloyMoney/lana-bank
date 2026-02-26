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

import { useDepositAccountCreateMutation } from "@/lib/graphql/generated"
import { useCreateContext } from "@/app/create"

gql`
  mutation DepositAccountCreate($input: DepositAccountCreateInput!) {
    depositAccountCreate(input: $input) {
      account {
        id
      }
    }
  }
`

type CreateDepositAccountDialogProps = {
  setOpenCreateDepositAccountDialog: (isOpen: boolean) => void
  openCreateDepositAccountDialog: boolean
  customerId: string
}

export const CreateDepositAccountDialog: React.FC<CreateDepositAccountDialogProps> = ({
  setOpenCreateDepositAccountDialog,
  openCreateDepositAccountDialog,
  customerId,
}) => {
  const t = useTranslations("Customers.CustomerDetails.createDepositAccount")
  const commonT = useTranslations("Common")
  const [createDepositAccount, { loading, reset }] = useDepositAccountCreateMutation()
  const [error, setError] = useState<string | null>(null)
  const { customer } = useCreateContext()

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    try {
      const result = await createDepositAccount({
        variables: {
          input: {
            customerId,
          },
        },
      })

      if (result.data?.depositAccountCreate) {
        toast.success(t("success"))
        handleCloseDialog()
      } else {
        setError(commonT("error"))
      }
    } catch (err) {
      setError(err instanceof Error && err.message ? err.message : commonT("error"))
    }
  }

  const handleCloseDialog = () => {
    setOpenCreateDepositAccountDialog(false)
    setError(null)
    reset()
  }

  return (
    <Dialog open={openCreateDepositAccountDialog} onOpenChange={handleCloseDialog}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("title")}</DialogTitle>
          <DialogDescription>
            {customer?.email &&
              t.rich("descriptionWithEmail", {
                b: (chunks) => <b>{chunks}</b>,
                email: customer.email,
              })}
          </DialogDescription>
        </DialogHeader>
        <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
          {error && <p className="text-destructive">{error}</p>}
          <DialogFooter>
            <Button
              type="submit"
              loading={loading}
              data-testid="create-deposit-account-dialog-button"
            >
              {t("buttons.submit")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

export default CreateDepositAccountDialog
