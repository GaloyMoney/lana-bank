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
  useCustomerFreezeMutation,
  GetCustomerBasicDetailsDocument,
  CustomerEventHistoryDocument,
} from "@/lib/graphql/generated"

gql`
  mutation CustomerFreeze($input: CustomerFreezeInput!) {
    customerFreeze(input: $input) {
      customer {
        id
        status
      }
    }
  }
`

type FreezeCustomerDialogProps = {
  setOpenFreezeDialog: (isOpen: boolean) => void
  openFreezeDialog: boolean
  customerId: string
}

export const FreezeCustomerDialog: React.FC<FreezeCustomerDialogProps> = ({
  setOpenFreezeDialog,
  openFreezeDialog,
  customerId,
}) => {
  const t = useTranslations("Customers.CustomerDetails.freezeCustomer")
  const commonT = useTranslations("Common")

  const [freezeCustomer, { loading, reset }] = useCustomerFreezeMutation({
    refetchQueries: [GetCustomerBasicDetailsDocument, CustomerEventHistoryDocument],
  })
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    try {
      const result = await freezeCustomer({
        variables: {
          input: { customerId },
        },
      })

      if (result.data) {
        toast.success(t("success"))
        handleCloseDialog()
      }
    } catch (error) {
      console.error("Error freezing customer:", error)
      setError(error instanceof Error ? error.message : commonT("error"))
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
          {error && <p className="text-destructive">{error}</p>}
          <DialogFooter>
            <Button
              data-testid="freeze-customer-dialog-button"
              type="submit"
              variant="destructive"
              disabled={loading}
            >
              {t("buttons.freeze")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

export default FreezeCustomerDialog
