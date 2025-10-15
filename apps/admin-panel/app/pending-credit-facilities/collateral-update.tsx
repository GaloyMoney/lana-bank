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

import { usePendingCreditFacilityCollateralUpdateMutation } from "@/lib/graphql/generated"
import { DetailItem, DetailsGroup } from "@/components/details"
import { currencyConverter, getCurrentLocalDate } from "@/lib/utils"
import Balance from "@/components/balance/balance"
import { Satoshis } from "@/types"

gql`
  mutation PendingCreditFacilityCollateralUpdate(
    $input: PendingCreditFacilityCollateralUpdateInput!
  ) {
    pendingCreditFacilityCollateralUpdate(input: $input) {
      pendingCreditFacility {
        id
        pendingCreditFacilityId
        collateral {
          btcBalance
        }
        collateralizationState
        ...PendingCreditFacilityLayoutFragment
      }
    }
  }
`

type PendingCreditFacilityCollateralUpdateDialogProps = {
  setOpenDialog: (isOpen: boolean) => void
  openDialog: boolean
  pendingCreditFacilityId: string
  currentCollateral: Satoshis
  collateralToMatchInitialCvl?: Satoshis | null
}

export const PendingCreditFacilityCollateralUpdateDialog: React.FC<
  PendingCreditFacilityCollateralUpdateDialogProps
> = ({
  setOpenDialog,
  openDialog,
  pendingCreditFacilityId,
  currentCollateral,
  collateralToMatchInitialCvl,
}) => {
  const t = useTranslations(
    "PendingCreditFacilities.PendingDetails.PendingCreditFacilityCollateralUpdate",
  )
  const commonT = useTranslations("Common")

  const [updateCollateral, { loading, reset }] =
    usePendingCreditFacilityCollateralUpdateMutation()
  const [error, setError] = useState<string | null>(null)
  const [isConfirmed, setIsConfirmed] = useState<boolean>(false)
  const [newCollateral, setNewCollateral] = useState<string>("")

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    try {
      const result = await updateCollateral({
        variables: {
          input: {
            pendingCreditFacilityId,
            collateral: currencyConverter.btcToSatoshi(Number(newCollateral)),
            effective: getCurrentLocalDate(),
          },
        },
      })
      if (result.data) {
        toast.success(t("messages.success"))
        handleCloseDialog()
      } else {
        setError(commonT("error"))
      }
    } catch (error) {
      console.error("Error updating pending credit facility collateral:", error)
      if (error instanceof Error) {
        setError(error.message)
      } else {
        setError(commonT("error"))
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

  return (
    <Dialog open={openDialog} onOpenChange={handleCloseDialog}>
      <DialogContent>
        {isConfirmed ? (
          <>
            <DialogHeader>
              <DialogTitle>{t("dialog.confirmTitle")}</DialogTitle>
              <DialogDescription>{t("dialog.confirmDescription")}</DialogDescription>
            </DialogHeader>
            <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
              <input
                type="text"
                className="sr-only"
                autoFocus
                onKeyDown={(e) => {
                  if (e.key === "Backspace") {
                    e.preventDefault()
                    setIsConfirmed(false)
                  }
                }}
              />
              <DetailsGroup layout="horizontal">
                <DetailItem
                  label={t("form.labels.currentCollateral")}
                  value={
                    <Balance amount={currentCollateral as Satoshis} currency="btc" />
                  }
                />
                <DetailItem
                  label={t("form.labels.newCollateral")}
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
                  type="button"
                  onClick={(e) => {
                    e.preventDefault()
                    setIsConfirmed(false)
                  }}
                  variant="ghost"
                  disabled={loading}
                >
                  {commonT("back")}
                </Button>
                <Button
                  type="submit"
                  loading={loading}
                  data-testid="confirm-update-button"
                >
                  {t("form.buttons.confirm")}
                </Button>
              </DialogFooter>
            </form>
          </>
        ) : (
          <>
            <DialogHeader>
              <DialogTitle>{t("dialog.title")}</DialogTitle>
              <DialogDescription>{t("dialog.description")}</DialogDescription>
            </DialogHeader>
            <form className="flex flex-col gap-4" onSubmit={handleConfirm}>
              <div className="rounded-md">
                <DetailsGroup layout="horizontal">
                  <DetailItem
                    label={t("form.labels.currentCollateral")}
                    value={
                      <Balance amount={currentCollateral as Satoshis} currency="btc" />
                    }
                    data-testid="current-collateral-balance"
                  />
                  {collateralToMatchInitialCvl && (
                    <DetailItem
                      label={t("form.labels.expectedCollateral")}
                      value={
                        <Balance amount={collateralToMatchInitialCvl} currency="btc" />
                      }
                      data-testid="expected-collateral-balance"
                    />
                  )}
                </DetailsGroup>
              </div>
              <div>
                <Label>{t("form.labels.newCollateral")}</Label>
                <div className="flex items-center gap-1">
                  <Input
                    autoFocus
                    type="number"
                    value={newCollateral}
                    onChange={(e) => setNewCollateral(e.target.value)}
                    placeholder={t("form.placeholders.newCollateral")}
                    step="0.00000001"
                    min="0"
                    required
                    data-testid="new-collateral-input"
                  />
                  <div className="p-1.5 bg-input-text rounded-md px-4">BTC</div>
                </div>
              </div>
              {error && <p className="text-destructive">{error}</p>}
              <DialogFooter>
                <Button
                  type="submit"
                  onClick={handleConfirm}
                  data-testid="proceed-to-confirm-button"
                >
                  {t("form.buttons.proceedToConfirm")}
                </Button>
              </DialogFooter>
            </form>
          </>
        )}
      </DialogContent>
    </Dialog>
  )
}
