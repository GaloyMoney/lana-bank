import { gql } from "@apollo/client"
import React, { useState } from "react"
import { toast } from "sonner"

import { useLoanPartialPaymentMutation } from "@/lib/graphql/generated"
import { currencyConverter } from "@/lib/utils"

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/primitive/dialog"
import { Input } from "@/components/primitive/input"
import { Label } from "@/components/primitive/label"
import { Button } from "@/components/primitive/button"

gql`
  mutation loanPartialPayment($input: LoanPartialPaymentInput!) {
    loanPartialPayment(input: $input) {
      loan {
        id
        loanId
        createdAt
        balance {
          collateral {
            btcBalance
          }
          outstanding {
            usdBalance
          }
          interestIncurred {
            usdBalance
          }
        }
      }
    }
  }
`

type LoanPartialPaymentDialogProps = {
  loanId: string
  refetch?: () => void
}

export const LoanPartialPaymentDialog: React.FC<
  React.PropsWithChildren<LoanPartialPaymentDialogProps>
> = ({ loanId, refetch, children }) => {
  const [loanIdValue, setLoanIdValue] = useState<string>(loanId)
  const [amount, setAmount] = useState<number>(0)
  const [open, setOpen] = useState<boolean>(false)
  const [loanPartialPayment, { loading, error, reset }] = useLoanPartialPaymentMutation()

  const handlePartialPaymentSubmit = async () => {
    try {
      await loanPartialPayment({
        variables: {
          input: {
            amount: currencyConverter.usdToCents(amount),
            loanId,
          },
        },
      })

      toast.success("Payment successful")
      if (refetch) refetch()
      setOpen(false)
      handleClose()
    } catch (error) {
      console.error(error)
    }
  }

  const handleClose = () => {
    setLoanIdValue(loanIdValue)
    setAmount(0)
    reset()
  }

  return (
    <Dialog
      open={open}
      onOpenChange={(isOpen) => {
        if (!isOpen) {
          handleClose()
        }
        setOpen(isOpen)
      }}
    >
      <DialogTrigger asChild>{children}</DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Loan Payment</DialogTitle>
          <DialogDescription>Fill in the details below.</DialogDescription>
        </DialogHeader>
        <div>
          <Label>Amount</Label>
          <div className="flex items-center gap-1">
            <Input
              type="number"
              value={amount}
              onChange={(e) => setAmount(Number(e.target.value))}
              placeholder="Enter the desired principal amount"
              min={0}
            />
            <div className="p-1.5 bg-input-text rounded-md px-4">USD</div>
          </div>
        </div>
        {error && <span className="text-destructive">{error.message}</span>}
        <DialogFooter className="mt-4">
          <Button
            className="w-32"
            disabled={loading}
            onClick={handlePartialPaymentSubmit}
          >
            Submit
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
