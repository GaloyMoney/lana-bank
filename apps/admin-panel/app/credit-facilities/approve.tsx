import React, { useState } from "react"
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
import {
  CreditFacility,
  GetCreditFacilityDetailsDocument,
  useApprovalProcessApproveMutation,
  useGetRealtimePriceUpdatesQuery,
} from "@/lib/graphql/generated"
import { DetailItem, DetailsGroup } from "@/components/details"
import Balance from "@/components/balance/balance"

type CreditFacilityApproveDialogProps = {
  setOpenDialog: (isOpen: boolean) => void
  openDialog: boolean
  creditFacilityDetails: CreditFacility
  onSuccess?: () => void
}

export const CreditFacilityApproveDialog: React.FC<CreditFacilityApproveDialogProps> = ({
  setOpenDialog,
  openDialog,
  creditFacilityDetails,
  onSuccess,
}) => {
  const [approveProcess, { loading, reset }] = useApprovalProcessApproveMutation({
    refetchQueries: [GetCreditFacilityDetailsDocument],
  })
  const [error, setError] = useState<string | null>(null)
  const { data: priceInfo } = useGetRealtimePriceUpdatesQuery({
    fetchPolicy: "cache-only",
  })

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    try {
      await approveProcess({
        variables: {
          input: {
            processId: creditFacilityDetails.approvalProcessId,
          },
        },
        onCompleted: (data) => {
          if (data.approvalProcessApprove) {
            toast.success("Credit facility approved successfully")
            if (onSuccess) onSuccess()
            handleCloseDialog()
          }
        },
      })
    } catch (error) {
      console.error("Error approving credit facility:", error)
      if (error instanceof Error) {
        setError(error.message)
      } else {
        setError("An unknown error occurred")
      }
    }
  }

  const handleCloseDialog = () => {
    setOpenDialog(false)
    setError(null)
    reset()
  }

  return (
    <Dialog open={openDialog} onOpenChange={handleCloseDialog}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Approve Credit Facility</DialogTitle>
          <DialogDescription>
            Are you sure you want to approve this credit facility?
          </DialogDescription>
        </DialogHeader>
        <DetailsGroup>
          <DetailItem
            label="Collateral Balance"
            className="px-0"
            value={
              <Balance
                amount={creditFacilityDetails.balance.collateral.btcBalance}
                currency="btc"
              />
            }
          />
          <DetailItem
            label="Expected Collateral to meet target CVL"
            className="px-0"
            value={
              <Balance
                amount={creditFacilityDetails.collateralToMatchInitialCvl}
                currency="btc"
              />
            }
          />
          <DetailItem
            className="px-0"
            label={
              <p className="text-textColor-secondary flex items-center">
                <div className="mr-2">Current CVL (BTC/USD:</div>
                <Balance
                  amount={priceInfo?.realtimePrice.usdCentsPerBtc}
                  currency="usd"
                />
                <div>)</div>
              </p>
            }
            value={`${creditFacilityDetails.currentCvl.total}%`}
          />
          <DetailItem
            className="px-0"
            label="Target (Initial) CVL"
            value={`${creditFacilityDetails.creditFacilityTerms.initialCvl}%`}
          />
          <DetailItem
            className="px-0"
            label="Margin Call CVL"
            value={`${creditFacilityDetails.creditFacilityTerms.marginCallCvl}%`}
          />
        </DetailsGroup>
        <form onSubmit={handleSubmit}>
          {error && <p className="text-destructive">{error}</p>}
          <DialogFooter className="mt-4">
            <Button type="button" variant="ghost" onClick={handleCloseDialog}>
              Cancel
            </Button>
            <Button type="submit" loading={loading}>
              Approve
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
