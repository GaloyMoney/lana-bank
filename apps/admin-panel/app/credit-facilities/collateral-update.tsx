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
import { Input } from "@/components/primitive/input"
import {
  GetCreditFacilityDetailsDocument,
  useCreditFacilityCollateralUpdateMutation,
  useGetCreditFacilityDetailsQuery,
} from "@/lib/graphql/generated"
import { DetailItem, DetailsGroup } from "@/components/details"
import { currencyConverter } from "@/lib/utils"
import Balance from "@/components/balance/balance"
import { Label } from "@/components/primitive/label"

gql`
  mutation CreditFacilityCollateralUpdate($input: CreditFacilityCollateralUpdateInput!) {
    creditFacilityCollateralUpdate(input: $input) {
      creditFacility {
        id
        creditFacilityId
        balance {
          collateral {
            btcBalance
          }
        }
      }
    }
  }
`

type CreditFacilityCollateralUpdateDialogProps = {
  setOpenDialog: (isOpen: boolean) => void
  openDialog: boolean
  creditFacilityId: string
  onSuccess?: () => void
}

export const CreditFacilityCollateralUpdateDialog: React.FC<
  CreditFacilityCollateralUpdateDialogProps
> = ({ setOpenDialog, openDialog, creditFacilityId, onSuccess }) => {
  const [updateCollateral, { loading, reset }] =
    useCreditFacilityCollateralUpdateMutation({
      refetchQueries: [GetCreditFacilityDetailsDocument],
    })
  const [error, setError] = useState<string | null>(null)
  const [isConfirmed, setIsConfirmed] = useState<boolean>(false)
  const [newCollateral, setNewCollateral] = useState<string>("")

  const { data: creditFacilityDetails } = useGetCreditFacilityDetailsQuery({
    variables: { id: creditFacilityId },
  })

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (newCollateral === "") {
      setError("Please enter a valid collateral amount.")
      return
    }
    setError(null)
    try {
      const result = await updateCollateral({
        variables: {
          input: {
            creditFacilityId,
            collateral: currencyConverter.btcToSatoshi(Number(newCollateral)),
          },
        },
      })
      if (result.data) {
        toast.success("Credit facility collateral updated successfully")
        handleCloseDialog()
        if (onSuccess) onSuccess()
      } else {
        throw new Error("No data returned from mutation")
      }
    } catch (error) {
      console.error("Error updating credit facility collateral:", error)
      if (error instanceof Error) {
        setError(error.message)
      } else {
        setError("An unknown error occurred")
      }
    }
  }

  const handleConfirm = () => {
    setIsConfirmed(true)
  }

  const handleCloseDialog = () => {
    setError(null)
    setIsConfirmed(false)
    reset()
    setOpenDialog(false)
    setNewCollateral("")
  }

  const currentCollateral =
    creditFacilityDetails?.creditFacility?.balance?.collateral?.btcBalance || 0

  return (
    <Dialog open={openDialog} onOpenChange={handleCloseDialog}>
      <DialogContent>
        {isConfirmed ? (
          <>
            <DialogHeader>
              <DialogTitle>Confirm Update</DialogTitle>
              <DialogDescription>
                Are you sure you want to update the collateral for this credit facility?
              </DialogDescription>
            </DialogHeader>
            <form className="flex flex-col gap-4 text-sm" onSubmit={handleSubmit}>
              <DetailsGroup>
                <DetailItem
                  className="text-sm"
                  label="Credit Facility ID"
                  value={creditFacilityId}
                />
                <DetailItem
                  className="text-sm"
                  label="Current Collateral"
                  value={<Balance amount={currentCollateral} currency="btc" />}
                />
                <DetailItem
                  className="text-sm"
                  label="New Collateral"
                  value={
                    <Balance
                      amount={currencyConverter.btcToSatoshi(Number(newCollateral))}
                      currency="btc"
                    />
                  }
                />
              </DetailsGroup>
              {error && <p className="text-destructive">{error}</p>}
              <DialogFooter>
                <Button
                  onClick={() => setIsConfirmed(false)}
                  variant="ghost"
                  className="text-primary"
                  disabled={loading}
                >
                  Back
                </Button>
                <Button type="submit" loading={loading}>
                  {loading ? "Updating..." : "Confirm"}
                </Button>
              </DialogFooter>
            </form>
          </>
        ) : (
          <>
            <DialogHeader>
              <DialogTitle>Update Credit Facility Collateral</DialogTitle>
              <DialogDescription>
                Enter the new collateral amount for this credit facility.
              </DialogDescription>
            </DialogHeader>
            <form className="flex flex-col gap-4" onSubmit={handleConfirm}>
              <div className="bg-secondary-foreground p-2 rounded-md text-sm">
                <DetailsGroup>
                  <DetailItem label="Credit Facility ID" value={creditFacilityId} />
                  <DetailItem
                    label="Current Collateral"
                    value={<Balance amount={currentCollateral} currency="btc" />}
                  />
                  <DetailItem
                    label="Expected Collateral"
                    value={
                      <Balance
                        amount={
                          creditFacilityDetails?.creditFacility
                            ?.collateralToMatchInitialCvl
                        }
                        currency="btc"
                      />
                    }
                  />
                </DetailsGroup>
              </div>
              <div>
                <Label>New Collateral</Label>
                <div className="flex items-center gap-1">
                  <Input
                    type="number"
                    value={newCollateral}
                    onChange={(e) => setNewCollateral(e.target.value)}
                    placeholder="Enter new collateral amount"
                    step="0.00000001"
                  />
                  <div className="p-1.5 bg-input-text rounded-md px-4">BTC</div>
                </div>
              </div>
              {error && <p className="text-destructive">{error}</p>}
              <DialogFooter>
                <Button type="submit" onClick={handleConfirm}>
                  Proceed to Confirm
                </Button>
              </DialogFooter>
            </form>
          </>
        )}
      </DialogContent>
    </Dialog>
  )
}
