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
import { useWithdrawalConfirmMutation } from "@/lib/graphql/generated"
import Balance from "@/components/balance/balance"
import { DetailItem, DetailsGroup } from "@/components/details"
import { currencyConverter, formatCurrency } from "@/lib/utils"

gql`
  mutation WithdrawalConfirm($input: WithdrawalConfirmInput!) {
    withdrawalConfirm(input: $input) {
      withdrawal {
        withdrawalId
        amount
        customer {
          customerId
          balance {
            checking {
              settled {
                amount
              }
              pending {
                amount
              }
            }
          }
        }
      }
    }
  }
`

export function WithdrawalConfirmDialog({
  setOpenWithdrawalConfirmDialog,
  openWithdrawalConfirmDialog,
  withdrawalData,
  refetch,
}: {
  setOpenWithdrawalConfirmDialog: (isOpen: boolean) => void
  openWithdrawalConfirmDialog: boolean
  withdrawalData: WithdrawalWithCustomer
  refetch?: () => void
}) {
  const [confirmWithdrawal, { loading, data, reset }] = useWithdrawalConfirmMutation()
  const [error, setError] = useState<string | null>(null)
  const [isConfirmed, setIsConfirmed] = useState(false)

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
      })
      if (result.data) {
        toast.success("Withdrawal confirmed successfully")
        setIsConfirmed(true)
        if (refetch) refetch()
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
    setIsConfirmed(false)
    reset()
  }

  return (
    <Dialog open={openWithdrawalConfirmDialog} onOpenChange={handleCloseDialog}>
      <DialogContent>
        {isConfirmed && data ? (
          <>
            <DialogHeader>
              <DialogTitle>Withdrawal Confirmed</DialogTitle>
              <DialogDescription>Details of the confirmed withdrawal.</DialogDescription>
            </DialogHeader>
            <div className="space-y-2">
              <DetailItem
                label="Withdrawal ID"
                value={data.withdrawalConfirm.withdrawal.withdrawalId}
              />
              <DetailItem
                label="Customer ID"
                value={data.withdrawalConfirm.withdrawal.customer?.customerId || "N/A"}
              />
              <DetailItem
                label="Amount"
                valueComponent={
                  <Balance
                    amount={data.withdrawalConfirm.withdrawal.amount}
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
              <DialogTitle>Confirm Withdrawal</DialogTitle>
              <DialogDescription>
                Are you sure you want to confirm this withdrawal?
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
                  value={formatCurrency({
                    currency: "USD",
                    amount: currencyConverter.centsToUsd(withdrawalData.amount),
                  })}
                />
              </DetailsGroup>
              {error && <p className="text-destructive">{error}</p>}
              <DialogFooter>
                <Button type="submit" disabled={loading}>
                  {loading ? "Confirming..." : "Confirm"}
                </Button>
              </DialogFooter>
            </form>
          </>
        )}
      </DialogContent>
    </Dialog>
  )
}
