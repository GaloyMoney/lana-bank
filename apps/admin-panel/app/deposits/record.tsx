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
  CustomersDocument,
  GetCustomerByCustomerEmailDocument,
  GetCustomerByCustomerIdDocument,
  useRecordDepositMutation,
} from "@/lib/graphql/generated"
import { currencyConverter } from "@/lib/utils"

gql`
  mutation RecordDeposit($input: DepositRecordInput!) {
    depositRecord(input: $input) {
      deposit {
        depositId
        amount
        customer {
          customerId
          email
          balance {
            checking {
              settled
            }
          }
        }
      }
    }
  }
`

type RecordDepositDialgProps = {
  setOpenRecordDepositDialog: (isOpen: boolean) => void
  openRecordDepositDialog: boolean
  customerId: string
  refetch?: () => void
}

export const RecordDepositDialog: React.FC<RecordDepositDialgProps> = ({
  setOpenRecordDepositDialog,
  openRecordDepositDialog,
  customerId,
  refetch,
}) => {
  const [recordDeposit, { loading, reset }] = useRecordDepositMutation()
  const [amount, setAmount] = useState<string>("")
  const [reference, setReference] = useState<string>("")
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    try {
      const result = await recordDeposit({
        variables: {
          input: {
            customerId,
            amount: currencyConverter.usdToCents(parseFloat(amount)),
            reference,
          },
        },
        refetchQueries: [
          GetCustomerByCustomerIdDocument,
          GetCustomerByCustomerEmailDocument,
          CustomersDocument,
        ],
      })
      if (result.data) {
        toast.success("Deposit recorded successfully")
        if (refetch) refetch()
        handleCloseDialog()
      } else {
        throw new Error("No data returned from mutation")
      }
    } catch (error) {
      console.error("Error recording deposit:", error)
      if (error instanceof Error) {
        setError(error.message)
      } else {
        setError("An unknown error occurred")
      }
    }
  }

  const resetStates = () => {
    setAmount("")
    setReference("")
    setError(null)
    reset()
  }

  const handleCloseDialog = () => {
    setOpenRecordDepositDialog(false)
    resetStates()
  }

  return (
    <Dialog open={openRecordDepositDialog} onOpenChange={handleCloseDialog}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Record Deposit</DialogTitle>
          <DialogDescription>
            Provide the required details to record a deposit.
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
                placeholder="Enter the deposit amount"
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
              {loading ? "Submitting..." : "Submit"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
