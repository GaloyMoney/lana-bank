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
  CustomersDocument,
  GetWithdrawalDetailsDocument,
  useWithdrawalConfirmMutation,
  WithdrawalsDocument,
} from "@/lib/graphql/generated"
import { DetailItem, DetailsGroup } from "@/components/details"
import Balance from "@/components/balance/balance"
import { UsdCents } from "@/types"

gql`
  mutation WithdrawalConfirm($input: WithdrawalConfirmInput!) {
    withdrawalConfirm(input: $input) {
      withdrawal {
        withdrawalId
        amount
        reference
        customer {
          customerId
          email
          balance {
            checking {
              settled
              pending
            }
          }
        }
      }
    }
  }
`

type WithdrawalConfirmDialogProps = {
  setOpenWithdrawalConfirmDialog: (isOpen: boolean) => void
  openWithdrawalConfirmDialog: boolean
  withdrawalData: WithdrawalWithCustomer
  refetch?: () => void
}

export const WithdrawalConfirmDialog: React.FC<WithdrawalConfirmDialogProps> = ({
  setOpenWithdrawalConfirmDialog,
  openWithdrawalConfirmDialog,
  withdrawalData,
  refetch,
}) => {
  const [confirmWithdrawal, { loading, reset }] = useWithdrawalConfirmMutation({
    refetchQueries: [WithdrawalsDocument, GetWithdrawalDetailsDocument],
  })
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    try {
      const result = await confirmWithdrawal({
        variables: {
          input: {
            withdrawalId: withdrawalData.withdrawalId,
          },
        },
        refetchQueries: [CustomersDocument, WithdrawalsDocument],
      })
      if (result.data) {
        toast.success("Withdrawal confirmed successfully")
        if (refetch) refetch()
        handleCloseDialog()
      } else {
        throw new Error("No data returned from mutation")
      }
    } catch (error) {
      console.error("Error confirming withdrawal:", error)
      if (error instanceof Error) {
        setError(error.message)
      } else {
        setError("An unknown error occurred")
      }
    }
  }

  const handleCloseDialog = () => {
    setOpenWithdrawalConfirmDialog(false)
    setError(null)
    reset()
  }

  return (
    <Dialog open={openWithdrawalConfirmDialog} onOpenChange={handleCloseDialog}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Confirm Withdrawal</DialogTitle>
          <DialogDescription>
            Are you sure you want to confirm this withdrawal?
          </DialogDescription>
        </DialogHeader>
        <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
          <DetailsGroup layout="horizontal">
            <DetailItem
              className="text-sm"
              label="Withdrawal ID"
              value={withdrawalData.withdrawalId}
            />
            <DetailItem
              className="text-sm"
              label="Customer Email"
              value={withdrawalData.customer?.email || "N/A"}
            />
            <DetailItem
              className="text-sm"
              label="Amount"
              value={
                <Balance amount={withdrawalData.amount as UsdCents} currency="usd" />
              }
            />
            <DetailItem
              className="text-sm"
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
            <Button type="submit" disabled={loading}>
              {loading ? "Confirming..." : "Confirm"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
