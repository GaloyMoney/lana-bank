"use client"

import React from "react"
import { useTranslations } from "next-intl"
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@lana/web/ui/dialog"

import { Label } from "@lana/web/ui/label"

import { ExternalLinkIcon } from "lucide-react"

import {
  GetCreditFacilityLayoutDetailsQuery,
  WalletNetwork,
} from "@/lib/graphql/generated"
import { DetailsCard, DetailItemProps } from "@/components/details"

type CreditFacilityWalletDialogProps = {
  openWalletDialog: boolean
  setOpenWalletDialog: (isOpen: boolean) => void
  wallet: NonNullable<
    NonNullable<GetCreditFacilityLayoutDetailsQuery["creditFacilityByPublicId"]>["wallet"]
  >
}

export const CreditFacilityWalletDialog: React.FC<CreditFacilityWalletDialogProps> = ({
  openWalletDialog,
  setOpenWalletDialog,
  wallet,
}) => {
  const t = useTranslations("CreditFacilities.CreditFacilityDetails.WalletDialog")

  const details: DetailItemProps[] = [
    {
      label: (
        <Label className="inline-flex items-center">
          {t("details.address")}
          <a
            href={mempoolAddressUrl(wallet.address, wallet.network)}
            target="_blank"
            className="ml-2 inline-flex items-center gap-1 text-xs text-purple-500 whitespace-nowrap leading-none"
            onClick={(e) => e.stopPropagation()}
          >
            <span className="leading-none">{t("details.viewOnMempool")}</span>
            <ExternalLinkIcon className="h-2.5 w-2.5 shrink-0" aria-hidden="true" />
          </a>
        </Label>
      ),
      value: wallet.address,
      className: "break-all",
    },
    {
      label: t("details.network"),
      value: wallet.network,
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

const MEMPOOL_BASE = {
  MAINNET: "https://mempool.space/address",
  TESTNET_3: "https://mempool.space/testnet/address",
  TESTNET_4: "https://mempool.space/testnet4/address",
} satisfies Record<WalletNetwork, string>

export function mempoolAddressUrl(address: string, network: WalletNetwork) {
  return `${MEMPOOL_BASE[network]}/${encodeURIComponent(address)}`
}
