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
import { DetailItem, DetailsGroup } from "@/components/details"
import { useWithdrawalInitiateMutation } from "@/lib/graphql/generated"
import { currencyConverter } from "@/lib/utils"
import Balance from "@/components/balance/balance"

gql`
  mutation WithdrawalInitiate($input: WithdrawalInitiateInput!) {
    withdrawalInitiate(input: $input) {
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

function WithdrawalInitiateDialog({
  setOpenWithdrawalInitiateDialog,
  openWithdrawalInitiateDialog,
  customerId,
  refetch,
}: {
  setOpenWithdrawalInitiateDialog: (isOpen: boolean) => void
  openWithdrawalInitiateDialog: boolean
  customerId: string
  refetch?: () => void
}) {
  const [initiateWithdrawal, { loading, reset, data }] = useWithdrawalInitiateMutation()
  const [amount, setAmount] = useState<string>("")
  const [reference, setReference] = useState<string>("")
  const [error, setError] = useState<string | null>(null)
  const [isInitiated, setIsInitiated] = useState(false)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    try {
      const result = await initiateWithdrawal({
        variables: {
          input: {
            customerId,
            amount: currencyConverter.usdToCents(parseFloat(amount)),
            reference,
          },
        },
      })
      if (result.data) {
        toast.success("Withdrawal initiated successfully")
        setIsInitiated(true)
        if (refetch) refetch()
      } else {
        throw new Error("No data returned from mutation")
      }
    } catch (error) {
      console.error("Error initiating withdrawal:", error)
      if (error instanceof Error) {
        setError(error.message)
      } else {
        setError("An unknown error occurred")
      }
    }
  }

  const handleCloseDialog = () => {
    setOpenWithdrawalInitiateDialog(false)
    setAmount("")
    setReference("")
    setError(null)
    setIsInitiated(false)
    reset()
  }

  return (
    <Dialog open={openWithdrawalInitiateDialog} onOpenChange={handleCloseDialog}>
      <DialogContent>
        {isInitiated && data ? (
          <>
            <DialogHeader>
              <DialogTitle>Withdrawal Initiated</DialogTitle>
              <DialogDescription>Details of the initiated withdrawal.</DialogDescription>
            </DialogHeader>
            <DetailsGroup>
              <DetailItem
                label="Withdrawal ID"
                value={data.withdrawalInitiate.withdrawal.withdrawalId}
              />
              <DetailItem
                label="Customer ID"
                value={data.withdrawalInitiate.withdrawal.customer?.customerId || "N/A"}
              />
              <DetailItem
                label="Amount"
                valueComponent={
                  <Balance
                    amount={data.withdrawalInitiate.withdrawal.amount}
                    currency="usd"
                  />
                }
              />
            </DetailsGroup>
            <DialogFooter>
              <Button onClick={handleCloseDialog}>Close</Button>
            </DialogFooter>
          </>
        ) : (
          <>
            <DialogHeader>
              <DialogTitle>Initiate Withdrawal</DialogTitle>
              <DialogDescription>
                Provide the required details to initiate a withdrawal.
              </DialogDescription>
            </DialogHeader>
            <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
              <div>
                <Label htmlFor="amount">Amount</Label>
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
              <div>
                <Label htmlFor="reference">Reference</Label>
                <Input
                  id="reference"
                  type="text"
                  placeholder="Enter a reference (optional)"
                  value={reference}
                  onChange={(e) => setReference(e.target.value)}
                />
              </div>
              {error && <p className="text-destructive">{error}</p>}
              <DialogFooter>
                <Button type="submit" disabled={loading}>
                  {loading ? "Initiating..." : "Initiate Withdrawal"}
                </Button>
              </DialogFooter>
            </form>
          </>
        )}
      </DialogContent>
    </Dialog>
  )
}

export default WithdrawalInitiateDialog
