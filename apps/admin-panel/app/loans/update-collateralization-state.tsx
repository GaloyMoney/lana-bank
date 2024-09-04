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
} from "@/components/primitive/dialog"
import { Button } from "@/components/primitive/button"
import { useCollateralizationStateUpdateMutation } from "@/lib/graphql/generated"
import { DetailItem, DetailsGroup } from "@/components/details"

gql`
  mutation CollateralizationStateUpdate($input: CollateralizationStateUpdateInput!) {
    collateralizationStateUpdate(input: $input) {
      loan {
        loanId
        collateralizationState
      }
    }
  }
`

type CollateralizationStateUpdateDialogProps = {
  setOpenDialog: (isOpen: boolean) => void
  openDialog: boolean
  loanData: {
    loanId: string
    currentState: string
  }
  refetch?: () => void
}

export const CollateralizationStateUpdateDialog: React.FC<
  CollateralizationStateUpdateDialogProps
> = ({ setOpenDialog, openDialog, loanData, refetch }) => {
  const [updateCollateralizationState, { loading, reset }] =
    useCollateralizationStateUpdateMutation()
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    try {
      const result = await updateCollateralizationState({
        variables: {
          input: {
            loanId: loanData.loanId,
          },
        },
      })
      if (result.data) {
        toast.success("Collateralization state updated successfully")
        if (refetch) refetch()
        handleCloseDialog()
      } else {
        throw new Error("No data returned from mutation")
      }
    } catch (error) {
      console.error("Error updating collateralization state:", error)
      if (error instanceof Error) {
        setError(error.message)
      } else {
        setError("An unknown error occurred")
      }
    }
  }

  const handleCloseDialog = () => {
    setOpenDialog(false)
    setError(null)
    reset()
  }

  return (
    <Dialog
      open={openDialog}
      onOpenChange={(isOpen) => {
        if (!isOpen) {
          handleCloseDialog()
        }
      }}
    >
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Update Collateralization State</DialogTitle>
          <DialogDescription>
            Confirm to update the collateralization state for this loan.
          </DialogDescription>
        </DialogHeader>
        <DetailsGroup>
          <DetailItem label="Loan ID" value={loanData.loanId} />
          <DetailItem label="Current State" value={loanData.currentState} />
        </DetailsGroup>
        {error && <p className="text-destructive">{error}</p>}
        <DialogFooter>
          <Button loading={loading} onClick={handleSubmit}>
            Submit
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
