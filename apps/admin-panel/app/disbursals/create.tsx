import React, { useState } from "react"
import { gql } from "@apollo/client"
import { toast } from "sonner"
import { useRouter } from "next/navigation"

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@lana/web/ui/dialog"
import { Input } from "@lana/web/ui/input"
import { Button } from "@lana/web/ui/button"
import { Label } from "@lana/web/ui/label"

import { useCreditFacilityDisbursalInitiateMutation } from "@/lib/graphql/generated"
import { currencyConverter } from "@/lib/utils"

gql`
  mutation CreditFacilityDisbursalInitiate(
    $input: CreditFacilityDisbursalInitiateInput!
  ) {
    creditFacilityDisbursalInitiate(input: $input) {
      disbursal {
        id
        disbursalId
        index
        amount
        status
        createdAt
        creditFacility {
          id
          disbursals {
            ...DisbursalOnFacilityPage
          }
          ...CreditFacilityTransactionsFragment
          ...CreditFacilityBasicDetailsFragment
          ...CreditFacilityOverviewFragment
        }
      }
    }
  }
`

type CreditFacilityDisbursalInitiateDialogProps = {
  setOpenDialog: (isOpen: boolean) => void
  openDialog: boolean
  creditFacilityId: string
}

export const CreditFacilityDisbursalInitiateDialog: React.FC<
  CreditFacilityDisbursalInitiateDialogProps
> = ({ setOpenDialog, openDialog, creditFacilityId }) => {
  const router = useRouter()
  const [initiateDisbursal, { loading, reset }] =
    useCreditFacilityDisbursalInitiateMutation({
      update: (cache) => {
        cache.modify({
          fields: {
            disbursals: (_, { DELETE }) => DELETE,
          },
        })
        cache.gc()
      },
    })
  const [amount, setAmount] = useState<string>("")
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    try {
      await initiateDisbursal({
        variables: {
          input: {
            creditFacilityId,
            amount: currencyConverter.usdToCents(parseFloat(amount)),
          },
        },
        onCompleted: (data) => {
          if (data.creditFacilityDisbursalInitiate) {
            router.push(
              `/disbursals/${data.creditFacilityDisbursalInitiate.disbursal.disbursalId}`,
            )
            toast.success("Disbursal initiated successfully")
            handleCloseDialog()
          }
        },
      })
    } catch (error) {
      console.error("Error initiating disbursal:", error)
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
      <DialogContent data-testid="disbursal-dialog-content">
        <DialogHeader>
          <DialogTitle>Initiate Credit Facility Disbursal</DialogTitle>
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
                data-testid="disbursal-amount-input"
              />
              <div className="p-1.5 bg-input-text rounded-md px-4">USD</div>
            </div>
          </div>
          {error && <p className="text-destructive mb-4">{error}</p>}
          <DialogFooter>
            <Button type="button" variant="ghost" onClick={handleCloseDialog}>
              Cancel
            </Button>
            <Button type="submit" loading={loading} data-testid="disbursal-submit-button">
              Initiate Disbursal
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
