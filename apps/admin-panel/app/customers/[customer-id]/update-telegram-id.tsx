import React, { useState } from "react"
import { gql } from "@apollo/client"
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

import { useCustomerUpdateMutation } from "@/lib/graphql/generated"

gql`
  mutation CustomerUpdate($input: CustomerUpdateInput!) {
    customerUpdate(input: $input) {
      customer {
        id
        telegramId
      }
    }
  }
`

type UpdateTelegramIdDialogProps = {
  setOpenUpdateTelegramIdDialog: (isOpen: boolean) => void
  openUpdateTelegramIdDialog: boolean
  customerId: string
}

export const UpdateTelegramIdDialog: React.FC<UpdateTelegramIdDialogProps> = ({
  setOpenUpdateTelegramIdDialog,
  openUpdateTelegramIdDialog,
  customerId,
}) => {
  const [updateTelegramId, { loading, error: mutationError, reset }] =
    useCustomerUpdateMutation()
  const [newTelegramId, setNewTelegramId] = useState<string>("")
  const [validationError, setValidationError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setValidationError(null)

    if (!newTelegramId.trim()) {
      setValidationError("Telegram ID cannot be empty")
      return
    }

    try {
      await updateTelegramId({
        variables: {
          input: {
            customerId,
            telegramId: newTelegramId.trim(),
          },
        },
      })
      toast.success("Telegram ID updated successfully")
      setOpenUpdateTelegramIdDialog(false)
    } catch (error) {
      console.error(error)
      if (error instanceof Error) {
        toast.error(`Failed to update Telegram ID: ${error.message}`)
      } else {
        toast.error("An unexpected error occurred")
      }
    }
  }

  const resetStates = () => {
    setNewTelegramId("")
    setValidationError(null)
    reset()
  }

  return (
    <Dialog
      open={openUpdateTelegramIdDialog}
      onOpenChange={(isOpen) => {
        setOpenUpdateTelegramIdDialog(isOpen)
        if (!isOpen) {
          resetStates()
        }
      }}
    >
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Update Telegram ID</DialogTitle>
          <DialogDescription>Update the Telegram ID for this customer</DialogDescription>
        </DialogHeader>
        <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
          <div>
            <Label htmlFor="newTelegramId">New Telegram ID</Label>
            <Input
              id="newTelegramId"
              type="text"
              required
              placeholder="Please enter the new Telegram ID"
              value={newTelegramId}
              onChange={(e) => setNewTelegramId(e.target.value)}
            />
          </div>
          {(validationError || mutationError) && (
            <p className="text-destructive">
              {validationError || mutationError?.message || "An error occurred"}
            </p>
          )}
          <DialogFooter>
            <Button type="submit" disabled={loading}>
              {loading ? "Updating..." : "Update"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

export default UpdateTelegramIdDialog
