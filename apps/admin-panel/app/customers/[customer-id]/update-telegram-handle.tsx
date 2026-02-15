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

import { useCustomerTelegramHandleUpdateMutation } from "@/lib/graphql/generated"

gql`
  mutation CustomerTelegramHandleUpdate($input: CustomerTelegramHandleUpdateInput!) {
    customerTelegramHandleUpdate(input: $input) {
      customer {
        id
        telegramHandle
      }
    }
  }
`

type UpdateTelegramHandleDialogProps = {
  setOpenUpdateTelegramHandleDialog: (isOpen: boolean) => void
  openUpdateTelegramHandleDialog: boolean
  customerId: string
}

export const UpdateTelegramHandleDialog: React.FC<UpdateTelegramHandleDialogProps> = ({
  setOpenUpdateTelegramHandleDialog,
  openUpdateTelegramHandleDialog,
  customerId,
}) => {
  const t = useTranslations("Customers.CustomerDetails.updateTelegram")

  const [updateTelegramHandle, { loading, error: mutationError, reset }] =
    useCustomerTelegramHandleUpdateMutation()
  const [newTelegramHandle, setNewTelegramHandle] = useState<string>("")
  const [validationError, setValidationError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setValidationError(null)

    if (!newTelegramHandle.trim()) {
      setValidationError(t("errors.emptyTelegramId"))
      return
    }

    try {
      await updateTelegramHandle({
        variables: {
          input: {
            customerId,
            telegramHandle: newTelegramHandle.trim(),
          },
        },
      })
      toast.success(t("messages.updateSuccess"))
      resetStates()
      setOpenUpdateTelegramHandleDialog(false)
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
    setNewTelegramHandle("")
    setValidationError(null)
    reset()
  }

  return (
    <Dialog
      open={openUpdateTelegramHandleDialog}
      onOpenChange={(isOpen) => {
        setOpenUpdateTelegramHandleDialog(isOpen)
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
            <Label htmlFor="newTelegramHandle">{t("labels.newTelegramId")}</Label>
            <Input
              id="newTelegramHandle"
              type="text"
              required
              placeholder={t("placeholders.newTelegramId")}
              value={newTelegramHandle}
              onChange={(e) => setNewTelegramHandle(e.target.value)}
            />
          </div>
          {(validationError || mutationError) && (
            <p className="text-destructive">
              {validationError || mutationError?.message || t("errors.unexpected")}
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

export default UpdateTelegramHandleDialog
