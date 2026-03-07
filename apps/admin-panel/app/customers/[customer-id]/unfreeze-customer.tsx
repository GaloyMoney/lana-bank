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
  useCustomerUnfreezeMutation,
  GetCustomerBasicDetailsDocument,
} from "@/lib/graphql/generated"

gql`
  mutation CustomerUnfreeze($input: CustomerUnfreezeInput!) {
    customerUnfreeze(input: $input) {
      customer {
        id
        status
      }
    }
  }
`

type UnfreezeCustomerDialogProps = {
  setOpenUnfreezeDialog: (isOpen: boolean) => void
  openUnfreezeDialog: boolean
  customerId: string
}

export const UnfreezeCustomerDialog: React.FC<UnfreezeCustomerDialogProps> = ({
  setOpenUnfreezeDialog,
  openUnfreezeDialog,
  customerId,
}) => {
  const t = useTranslations("Customers.CustomerDetails.unfreezeCustomer")
  const commonT = useTranslations("Common")

  const [unfreezeCustomer, { loading, reset }] = useCustomerUnfreezeMutation({
    refetchQueries: [GetCustomerBasicDetailsDocument],
  })
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    try {
      const result = await unfreezeCustomer({
        variables: {
          input: { customerId },
        },
      })

      if (result.data) {
        toast.success(t("success"))
        handleCloseDialog()
      }
    } catch (error) {
      console.error("Error unfreezing customer:", error)
      setError(error instanceof Error ? error.message : commonT("error"))
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
              data-testid="unfreeze-customer-dialog-button"
              type="submit"
              disabled={loading}
            >
              {t("buttons.unfreeze")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

export default UnfreezeCustomerDialog
