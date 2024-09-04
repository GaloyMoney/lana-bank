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
import { useWithdrawalCancelMutation } from "@/lib/graphql/generated"
import Balance from "@/components/balance/balance"
import { DetailItem, DetailsGroup } from "@/components/details"
import { currencyConverter } from "@/lib/utils"

gql`
  mutation WithdrawalCancel($input: WithdrawalCancelInput!) {
    withdrawalCancel(input: $input) {
      withdrawal {
        withdrawalId
        amount
        customer {
          customerId
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

type WithdrawalCancelDialogProps = {
  setOpenWithdrawalCancelDialog: (isOpen: boolean) => void
  openWithdrawalCancelDialog: boolean
  withdrawalData: WithdrawalWithCustomer
  refetch?: () => void
}

export const WithdrawalCancelDialog: React.FC<WithdrawalCancelDialogProps> = ({
  setOpenWithdrawalCancelDialog,
  openWithdrawalCancelDialog,
  withdrawalData,
  refetch,
}) => {
  const [cancelWithdrawal, { loading, data, reset }] = useWithdrawalCancelMutation()
  const [error, setError] = useState<string | null>(null)
  const [isCanceled, setIsCanceled] = useState(false)

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
        setIsCanceled(true)
        if (refetch) refetch()
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
    setIsCanceled(false)
    reset()
  }

  return (
    <Dialog open={openWithdrawalCancelDialog} onOpenChange={handleCloseDialog}>
      <DialogContent>
        {isCanceled && data ? (
          <>
            <DialogHeader>
              <DialogTitle>Withdrawal Canceled</DialogTitle>
              <DialogDescription>Details of the canceled withdrawal.</DialogDescription>
            </DialogHeader>
            <div className="space-y-2">
              <DetailItem
                label="Withdrawal ID"
                value={data.withdrawalCancel.withdrawal.withdrawalId}
              />
              <DetailItem
                label="Customer ID"
                value={data.withdrawalCancel.withdrawal.customer?.customerId || "N/A"}
              />
              <DetailItem
                label="Amount"
                valueComponent={
                  <Balance
                    amount={data.withdrawalCancel.withdrawal.amount}
                    currency="usd"
                  />
                }
              />
            </div>
            <DialogFooter>
              <Button onClick={handleCloseDialog}>Close</Button>
            </DialogFooter>
          </>
        ) : (
          <>
            <DialogHeader>
              <DialogTitle>Cancel Withdrawal</DialogTitle>
              <DialogDescription>
                Are you sure you want to cancel this withdrawal?
              </DialogDescription>
            </DialogHeader>
            <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
              <DetailsGroup>
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
                  valueComponent={
                    <Balance
                      amount={currencyConverter.centsToUsd(withdrawalData.amount)}
                      currency="usd"
                    />
                  }
                />
                <DetailItem
                  label="Withdrawal Reference"
                  value={
                    withdrawalData.reference === withdrawalData.withdrawalId
                      ? "n/a"
                      : withdrawalData.reference
                  }
                />
              </DetailsGroup>
              {error && <p className="text-destructive">{error}</p>}
              <DialogFooter>
                <Button type="submit" disabled={loading}>
                  {loading ? "Canceling..." : "Confirm"}
                </Button>
              </DialogFooter>
            </form>
          </>
        )}
      </DialogContent>
    </Dialog>
  )
}
