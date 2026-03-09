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
  useCustomerCloseMutation,
  GetCustomerBasicDetailsDocument,
  CustomerEventHistoryDocument,
} from "@/lib/graphql/generated"

gql`
  mutation CustomerClose($input: CustomerCloseInput!) {
    customerClose(input: $input) {
      customer {
        id
        status
      }
    }
  }
`

type CloseCustomerDialogProps = {
  setOpenCloseDialog: (isOpen: boolean) => void
  openCloseDialog: boolean
  customerId: string
}

export const CloseCustomerDialog: React.FC<CloseCustomerDialogProps> = ({
  setOpenCloseDialog,
  openCloseDialog,
  customerId,
}) => {
  const t = useTranslations("Customers.CustomerDetails.closeCustomer")
  const commonT = useTranslations("Common")

  const [closeCustomer, { loading, reset }] = useCustomerCloseMutation({
    refetchQueries: [GetCustomerBasicDetailsDocument, CustomerEventHistoryDocument],
  })
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    try {
      const result = await closeCustomer({
        variables: {
          input: { customerId },
        },
      })

      if (result.data) {
        toast.success(t("success"))
        handleCloseDialog()
      }
    } catch (error) {
      console.error("Error closing customer:", error)
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
            <Button type="submit" variant="destructive" disabled={loading}>
              {t("buttons.close")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

export default CloseCustomerDialog
