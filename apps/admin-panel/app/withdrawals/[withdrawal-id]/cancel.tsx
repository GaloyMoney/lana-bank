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
} from "@/ui/dialog"
import { Button } from "@/ui/button"
import {
  GetWithdrawalDetailsQuery,
  useWithdrawalCancelMutation,
} from "@/lib/graphql/generated"
import Balance from "@/components/balance/balance"
import { DetailItem, DetailsGroup } from "@/components/details"
import { UsdCents } from "@/types"

gql`
  mutation WithdrawalCancel($input: WithdrawalCancelInput!) {
    withdrawalCancel(input: $input) {
      withdrawal {
        ...WithdrawDetailsPageFragment
      }
    }
  }
`

type WithdrawalCancelDialogProps = {
  setOpenWithdrawalCancelDialog: (isOpen: boolean) => void
  openWithdrawalCancelDialog: boolean
  withdrawalData: NonNullable<GetWithdrawalDetailsQuery["withdrawal"]>
}

export const WithdrawalCancelDialog: React.FC<WithdrawalCancelDialogProps> = ({
  setOpenWithdrawalCancelDialog,
  openWithdrawalCancelDialog,
  withdrawalData,
}) => {
  const [cancelWithdrawal, { loading, reset }] = useWithdrawalCancelMutation()
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    try {
      const result = await cancelWithdrawal({
        variables: {
          input: {
            withdrawalId: withdrawalData.withdrawalId,
          },
        },
      })
      if (result.data) {
        toast.success("Withdrawal canceled successfully")
        handleCloseDialog()
      } else {
        throw new Error("No data returned from mutation")
      }
    } catch (error) {
      console.error("Error canceling withdrawal:", error)
      if (error instanceof Error) {
        setError(error.message)
      } else {
        setError("An unknown error occurred")
      }
    }
  }

  const handleCloseDialog = () => {
    setOpenWithdrawalCancelDialog(false)
    setError(null)
    reset()
  }

  return (
    <Dialog open={openWithdrawalCancelDialog} onOpenChange={handleCloseDialog}>
      <DialogContent>
        <>
          <DialogHeader>
            <DialogTitle>Cancel Withdrawal</DialogTitle>
            <DialogDescription>
              Are you sure you want to cancel this withdrawal?
            </DialogDescription>
          </DialogHeader>
          <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
            <DetailsGroup layout="horizontal">
              <DetailItem
                label="Customer Email"
                value={withdrawalData.account.customer?.email || "N/A"}
              />
              <DetailItem
                label="Amount"
                value={
                  <Balance amount={withdrawalData.amount as UsdCents} currency="usd" />
                }
              />
              <DetailItem
                label="Withdrawal Reference"
                value={
                  withdrawalData.reference === withdrawalData.withdrawalId
                    ? "N/A"
                    : withdrawalData.reference
                }
              />
            </DetailsGroup>
            {error && <p className="text-destructive">{error}</p>}
            <DialogFooter>
              <Button
                type="submit"
                disabled={loading}
                data-testid="withdrawal-confirm-dialog-button"
              >
                {loading ? "Canceling..." : "Confirm"}
              </Button>
            </DialogFooter>
          </form>
        </>
      </DialogContent>
    </Dialog>
  )
}
