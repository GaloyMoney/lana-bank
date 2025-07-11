"use client"

import React from "react"
import { useTranslations } from "next-intl"
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@lana/web/ui/dialog"

import { GetCreditFacilityLayoutDetailsQuery } from "@/lib/graphql/generated"
import { DetailsCard, DetailItemProps } from "@/components/details"

type CreditFacilityWalletDialogProps = {
  openWalletDialog: boolean
  setOpenWalletDialog: (isOpen: boolean) => void
  wallet: NonNullable<GetCreditFacilityLayoutDetailsQuery["creditFacility"]>["wallet"]
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
    },
    {
      label: t("details.address"),
      value: wallet?.address,
    },
  ]

  return (
    <Dialog open={openWalletDialog} onOpenChange={setOpenWalletDialog}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("title")}</DialogTitle>
        </DialogHeader>
        <DetailsCard columns={2} variant="container" details={details} />
      </DialogContent>
    </Dialog>
  )
}
