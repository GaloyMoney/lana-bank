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
import { Input } from "@/components/primitive/input"
import { Button } from "@/components/primitive/button"
import { Label } from "@/components/primitive/label"
import {
  GetCreditFacilityDetailsDocument,
  useCreditFacilityDisbursementInitiateMutation,
} from "@/lib/graphql/generated"
import { currencyConverter } from "@/lib/utils"

gql`
  mutation CreditFacilityDisbursementInitiate(
    $input: CreditFacilityDisbursementInitiateInput!
  ) {
    creditFacilityDisbursementInitiate(input: $input) {
      disbursement {
        id
        index
      }
    }
  }
`

type CreditFacilityDisbursementInitiateDialogProps = {
  setOpenDialog: (isOpen: boolean) => void
  openDialog: boolean
  creditFacilityId: string
  onSuccess?: () => void
}

export const CreditFacilityDisbursementInitiateDialog: React.FC<
  CreditFacilityDisbursementInitiateDialogProps
> = ({ setOpenDialog, openDialog, creditFacilityId, onSuccess }) => {
  const [initiateDisbursement, { loading, reset }] =
    useCreditFacilityDisbursementInitiateMutation({
      refetchQueries: [GetCreditFacilityDetailsDocument],
    })
  const [amount, setAmount] = useState<string>("")
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    try {
      await initiateDisbursement({
        variables: {
          input: {
            creditFacilityId,
            amount: currencyConverter.usdToCents(parseFloat(amount)),
          },
        },
        onCompleted: (data) => {
          if (data.creditFacilityDisbursementInitiate) {
            toast.success("Disbursement initiated successfully")
            if (onSuccess) onSuccess()
            handleCloseDialog()
          }
        },
      })
    } catch (error) {
      console.error("Error initiating disbursement:", error)
      if (error instanceof Error) {
        setError(error.message)
      } else {
        setError("An unknown error occurred")
      }
    }
  }

  const handleCloseDialog = () => {
    setOpenDialog(false)
    setAmount("")
    setError(null)
    reset()
  }

  return (
    <Dialog open={openDialog} onOpenChange={handleCloseDialog}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Initiate Credit Facility Disbursement</DialogTitle>
          <DialogDescription>
            Enter the amount you want to disburse from this credit facility.
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit}>
          <div className="mb-4">
            <Label htmlFor="amount">Amount (USD)</Label>
            <div className="flex items-center gap-1">
              <Input
                id="amount"
                type="number"
                required
                placeholder="Enter amount"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
              />
              <div className="p-1.5 bg-input-text rounded-md px-4">USD</div>
            </div>
          </div>
          {error && <p className="text-destructive mb-4">{error}</p>}
          <DialogFooter>
            <Button type="button" variant="ghost" onClick={handleCloseDialog}>
              Back
            </Button>
            <Button type="submit" loading={loading}>
              Initiate Disbursement
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
