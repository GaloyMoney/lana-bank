import React, { useState } from "react"
import { gql } from "@apollo/client"
import { toast } from "sonner"
import { useTranslations } from "next-intl"

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@lana/web/ui/dialog"
import { Button } from "@lana/web/ui/button"
import { Input } from "@lana/web/ui/input"

import { Label } from "@lana/web/ui/label"

import { useCreditFacilityProposalCollateralUpdateMutation } from "@/lib/graphql/generated"
import { DetailItem, DetailsGroup } from "@/components/details"
import { currencyConverter, getCurrentLocalDate } from "@/lib/utils"
import Balance from "@/components/balance/balance"
import { Satoshis } from "@/types"

gql`
  mutation CreditFacilityProposalCollateralUpdate($input: CreditFacilityProposalCollateralUpdateInput!) {
    creditFacilityProposalCollateralUpdate(input: $input) {
      creditFacilityProposal {
        id
        creditFacilityProposalId
        collateral {
          btcBalance
        }
      }
    }
  }
`

type CreditFacilityProposalCollateralUpdateDialogProps = {
  setOpenDialog: (isOpen: boolean) => void
  openDialog: boolean
  creditFacilityProposalId: string
  currentCollateral: Satoshis
}

export const CreditFacilityProposalCollateralUpdateDialog: React.FC<
  CreditFacilityProposalCollateralUpdateDialogProps
> = ({ setOpenDialog, openDialog, creditFacilityProposalId, currentCollateral }) => {
  const t = useTranslations(
    "CreditFacilityProposals.CreditFacilityProposalCollateralUpdate",
  )

  const [collateralUpdateMutation, { loading }] =
    useCreditFacilityProposalCollateralUpdateMutation()

  const [newCollateral, setNewCollateral] = useState("")
  const [confirming, setConfirming] = useState(false)

  const handleCloseDialog = () => {
    setOpenDialog(false)
    setNewCollateral("")
    setConfirming(false)
  }

  const handleProceedToConfirm = () => {
    if (!newCollateral) {
      toast.error(t("form.errors.emptyCollateral"))
      return
    }
    setConfirming(true)
  }

  const handleBackToForm = () => {
    setConfirming(false)
  }

  const handleCollateralUpdate = async () => {
    try {
      const response = await collateralUpdateMutation({
        variables: {
          input: {
            creditFacilityProposalId,
            collateral: currencyConverter.btcToSatoshis(Number(newCollateral)),
            effective: getCurrentLocalDate(),
          },
        },
      })

      if (!response.data) {
        toast.error(t("form.errors.noData"))
        return
      }

      toast.success(t("messages.success"))
      handleCloseDialog()
    } catch (err) {
      console.error("Error updating proposal collateral:", err)
      toast.error(t("form.errors.unknownError"))
    }
  }

  const currentCollateralBtc = currencyConverter.satoshisToBtc(currentCollateral)
  const newCollateralBtc = newCollateral ? Number(newCollateral) : 0

  return (
    <Dialog open={openDialog} onOpenChange={handleCloseDialog}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle>
            {confirming ? t("dialog.confirmTitle") : t("dialog.title")}
          </DialogTitle>
          <DialogDescription>
            {confirming ? t("dialog.confirmDescription") : t("dialog.description")}
          </DialogDescription>
        </DialogHeader>
        {!confirming ? (
          <div className="flex flex-col gap-4">
            <div>
              <Label htmlFor="currentCollateral">{t("form.labels.currentCollateral")}</Label>
              <div className="p-2 bg-muted rounded">
                <Balance amount={currentCollateralBtc} currency="btc" />
              </div>
            </div>
            <div>
              <Label htmlFor="newCollateral">{t("form.labels.newCollateral")}</Label>
              <div className="flex items-center gap-2">
                <Input
                  id="newCollateral"
                  type="number"
                  step="0.00000001"
                  min="0"
                  value={newCollateral}
                  onChange={(e) => setNewCollateral(e.target.value)}
                  placeholder={t("form.placeholders.newCollateral")}
                />
                <span className="text-sm text-muted-foreground">{t("units.btc")}</span>
              </div>
            </div>
          </div>
        ) : (
          <div className="flex flex-col gap-4">
            <DetailsGroup>
              <DetailItem label={t("form.labels.currentCollateral")}>
                <Balance amount={currentCollateralBtc} currency="btc" />
              </DetailItem>
              <DetailItem label={t("form.labels.newCollateral")}>
                <Balance amount={newCollateralBtc} currency="btc" />
              </DetailItem>
            </DetailsGroup>
          </div>
        )}
        <DialogFooter>
          {!confirming ? (
            <>
              <Button variant="outline" onClick={handleCloseDialog}>
                {t("form.buttons.back")}
              </Button>
              <Button onClick={handleProceedToConfirm}>
                {t("form.buttons.proceedToConfirm")}
              </Button>
            </>
          ) : (
            <>
              <Button variant="outline" onClick={handleBackToForm}>
                {t("form.buttons.back")}
              </Button>
              <Button onClick={handleCollateralUpdate} disabled={loading}>
                {loading ? t("form.buttons.updating") : t("form.buttons.confirm")}
              </Button>
            </>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}