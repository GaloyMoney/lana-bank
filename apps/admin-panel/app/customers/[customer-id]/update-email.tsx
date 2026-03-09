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
import { Input } from "@lana/web/ui/input"
import { Button } from "@lana/web/ui/button"
import { Label } from "@lana/web/ui/label"

import {
  useCustomerEmailUpdateMutation,
  CustomerEventHistoryDocument,
} from "@/lib/graphql/generated"

gql`
  mutation CustomerEmailUpdate($input: CustomerEmailUpdateInput!) {
    customerEmailUpdate(input: $input) {
      customer {
        id
        email
      }
    }
  }
`

type UpdateEmailDialogProps = {
  setOpenUpdateEmailDialog: (isOpen: boolean) => void
  openUpdateEmailDialog: boolean
  customerId: string
}

export const UpdateEmailDialog: React.FC<UpdateEmailDialogProps> = ({
  setOpenUpdateEmailDialog,
  openUpdateEmailDialog,
  customerId,
}) => {
  const t = useTranslations("Customers.CustomerDetails.updateEmail")

  const [updateEmail, { loading, error: mutationError, reset }] =
    useCustomerEmailUpdateMutation({
      refetchQueries: [CustomerEventHistoryDocument],
    })
  const [newEmail, setNewEmail] = useState<string>("")
  const [validationError, setValidationError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setValidationError(null)

    if (!newEmail.trim()) {
      setValidationError(t("errors.emptyEmail"))
      return
    }

    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/
    if (!emailRegex.test(newEmail.trim())) {
      setValidationError(t("errors.invalidEmail"))
      return
    }

    try {
      await updateEmail({
        variables: {
          input: {
            customerId,
            email: newEmail.trim(),
          },
        },
      })
      toast.success(t("messages.updateSuccess"))
      resetStates()
      setOpenUpdateEmailDialog(false)
    } catch (error) {
      console.error(error)
      if (error instanceof Error) {
        toast.error(t("errors.updateFailed", { error: error.message }))
      } else {
        toast.error(t("errors.unexpected"))
      }
    }
  }

  const resetStates = () => {
    setNewEmail("")
    setValidationError(null)
    reset()
  }

  return (
    <Dialog
      open={openUpdateEmailDialog}
      onOpenChange={(isOpen) => {
        setOpenUpdateEmailDialog(isOpen)
        if (!isOpen) {
          resetStates()
        }
      }}
    >
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("title")}</DialogTitle>
          <DialogDescription>{t("description")}</DialogDescription>
        </DialogHeader>
        <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
          <div>
            <Label htmlFor="newEmail">{t("labels.newEmail")}</Label>
            <Input
              id="newEmail"
              type="email"
              required
              placeholder={t("placeholders.newEmail")}
              value={newEmail}
              onChange={(e) => setNewEmail(e.target.value)}
            />
          </div>
          {(validationError || mutationError) && (
            <p className="text-destructive">
              {validationError || mutationError?.message}
            </p>
          )}
          <DialogFooter>
            <Button type="submit" disabled={loading}>
              {loading ? t("actions.updating") : t("actions.update")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

export default UpdateEmailDialog
