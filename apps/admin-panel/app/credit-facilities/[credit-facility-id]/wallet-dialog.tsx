"use client"

import React from "react"
import { useTranslations } from "next-intl"
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@lana/web/ui/dialog"

import { GetCreditFacilityLayoutDetailsQuery } from "@/lib/graphql/generated"
import { DetailsCard, DetailItemProps } from "@/components/details"

type CreditFacilityWalletDialogProps = {
  openWalletDialog: boolean
  setOpenWalletDialog: (isOpen: boolean) => void
  wallet: NonNullable<
    GetCreditFacilityLayoutDetailsQuery["creditFacilityByPublicId"]
  >["wallet"]
}

export const CreditFacilityWalletDialog: React.FC<CreditFacilityWalletDialogProps> = ({
  openWalletDialog,
  setOpenWalletDialog,
  wallet,
}) => {
  const t = useTranslations("CreditFacilities.CreditFacilityDetails.WalletDialog")

  const details: DetailItemProps[] = [
    {
      label: t("details.walletId"),
      value: wallet?.walletId,
      className: "break-all",
    },
    {
      label: t("details.address"),
      value: wallet?.address,
      className: "break-all",
    },
  ]

  return (
    <Dialog open={openWalletDialog} onOpenChange={setOpenWalletDialog}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("title")}</DialogTitle>
        </DialogHeader>
        <DetailsCard columns={1} variant="container" details={details} />
      </DialogContent>
    </Dialog>
  )
}
