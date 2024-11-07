import React, { useState } from "react"
import { gql } from "@apollo/client"
import { toast } from "sonner"
import { useRouter } from "next/navigation"

import { useCreateContext } from "../create"

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
import { CustomersDocument, useWithdrawalInitiateMutation } from "@/lib/graphql/generated"
import { currencyConverter } from "@/lib/utils"

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

type WithdrawalInitiateDialogProps = {
  setOpenWithdrawalInitiateDialog: (isOpen: boolean) => void
  openWithdrawalInitiateDialog: boolean
  customerId: string
}

export const WithdrawalInitiateDialog: React.FC<WithdrawalInitiateDialogProps> = ({
  setOpenWithdrawalInitiateDialog,
  openWithdrawalInitiateDialog,
  customerId,
}) => {
  const router = useRouter()
  const { customer } = useCreateContext()

  const [initiateWithdrawal, { loading, reset }] = useWithdrawalInitiateMutation()
  const [amount, setAmount] = useState<string>("")
  const [reference, setReference] = useState<string>("")
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    try {
      await initiateWithdrawal({
        variables: {
          input: {
            customerId,
            amount: currencyConverter.usdToCents(parseFloat(amount)),
            reference,
          },
        },
        refetchQueries: [CustomersDocument],
        onCompleted: (data) => {
          toast.success("Withdrawal initiated successfully")
          router.push(`/withdrawals/${data.withdrawalInitiate.withdrawal.withdrawalId}`)
          handleCloseDialog()
        },
      })
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
    reset()
  }

  return (
    <Dialog open={openWithdrawalInitiateDialog} onOpenChange={handleCloseDialog}>
      <DialogContent>
        <div
          className="absolute -top-4 -left-[1px] bg-primary rounded-tl-md rounded-tr-md text-md px-2 py-1 text-secondary"
          style={{ width: "100.35%" }}
        >
          Creating withdrawal for {customer?.email}
        </div>
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
      </DialogContent>
    </Dialog>
  )
}
