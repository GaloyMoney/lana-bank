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
  useCreditFacilityDisbursalConfirmMutation,
} from "@/lib/graphql/generated"
import Balance from "@/components/balance/balance"
import { formatDate } from "@/lib/utils"
import { DetailItem, DetailsGroup } from "@/components/details"

gql`
  mutation CreditFacilityDisbursalConfirm(
    $input: CreditFacilityDisbursalConfirmInput!
  ) {
    creditFacilityDisbursalConfirm(input: $input) {
      disbursement {
        id
        index
      }
    }
  }
`

type CreditFacilityDisbursalConfirmDialogProps = {
  setOpenDialog: (isOpen: boolean) => void
  openDialog: boolean
  creditFacilityId: string
  disbursementIdx: number
  disbursement: {
    id: string
    index: number
    amount: number
    status: string
    createdAt: string
  }
  onSuccess?: () => void
}

export const CreditFacilityDisbursalConfirmDialog: React.FC<
  CreditFacilityDisbursalConfirmDialogProps
> = ({
  setOpenDialog,
  openDialog,
  creditFacilityId,
  disbursementIdx,
  disbursement,
  onSuccess,
}) => {
  const [confirmDisbursal, { loading, reset }] =
    useCreditFacilityDisbursalConfirmMutation({
      refetchQueries: [GetCreditFacilityDetailsDocument],
    })
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    try {
      await confirmDisbursal({
        variables: {
          input: {
            creditFacilityId,
            disbursementIdx,
          },
        },
        onCompleted: (data) => {
          if (data.creditFacilityDisbursalConfirm) {
            toast.success("Disbursal confirmed successfully")
            if (onSuccess) onSuccess()
            handleCloseDialog()
          }
        },
      })
    } catch (error) {
      console.error("Error confirming disbursement:", error)
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
          <DialogTitle>Confirm Credit Facility Disbursal</DialogTitle>
          <DialogDescription>
            Review the disbursement details before confirming.
          </DialogDescription>
        </DialogHeader>
        <DetailsGroup>
          <DetailItem
            className="px-0"
            label="ID"
            value={disbursement.id.split("disbursement:")[1]}
          />
          <DetailItem
            className="px-0"
            label="Amount"
            value={<Balance amount={disbursement.amount} currency="usd" />}
          />
          <DetailItem
            className="px-0"
            label="Created"
            value={formatDate(disbursement.createdAt)}
          />
        </DetailsGroup>
        <form onSubmit={handleSubmit}>
          {error && <p className="text-destructive mb-4">{error}</p>}
          <DialogFooter>
            <Button type="button" variant="ghost" onClick={handleCloseDialog}>
              Cancel
            </Button>
            <Button type="submit" disabled={loading}>
              {loading ? "Confirming..." : "Confirm Disbursal"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
