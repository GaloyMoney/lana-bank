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
import {
  GetCreditFacilityDetailsDocument,
  useCreditFacilityCompleteMutation,
} from "@/lib/graphql/generated"

gql`
  mutation CreditFacilityComplete($input: CreditFacilityCompleteInput!) {
    creditFacilityComplete(input: $input) {
      creditFacility {
        id
        creditFacilityId
      }
    }
  }
`

type CreditFacilityCompleteDialogProps = {
  setOpenDialog: (isOpen: boolean) => void
  openDialog: boolean
  creditFacilityId: string
  onSuccess?: () => void
}

export const CreditFacilityCompleteDialog: React.FC<
  CreditFacilityCompleteDialogProps
> = ({ setOpenDialog, openDialog, creditFacilityId, onSuccess }) => {
  const [completeCreditFacility, { loading, reset }] = useCreditFacilityCompleteMutation({
    refetchQueries: [GetCreditFacilityDetailsDocument],
  })
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    try {
      await completeCreditFacility({
        variables: {
          input: {
            creditFacilityId,
          },
        },
        onCompleted: (data) => {
          if (data.creditFacilityComplete) {
            toast.success("Credit facility completed successfully")
            if (onSuccess) onSuccess()
            handleCloseDialog()
          }
        },
      })
    } catch (error) {
      console.error("Error completing credit facility:", error)
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
    <Dialog open={openDialog} onOpenChange={handleCloseDialog}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Complete Credit Facility</DialogTitle>
          <DialogDescription>
            Are you sure you want to complete this credit facility? This action cannot be
            undone.
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit}>
          {error && <p className="text-destructive">{error}</p>}
          <DialogFooter>
            <Button type="button" variant="ghost" onClick={handleCloseDialog}>
              Cancel
            </Button>
            <Button type="submit" loading={loading}>
              Complete
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
