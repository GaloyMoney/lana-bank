"use client"

import React, { useState } from "react"
import { useTranslations } from "next-intl"
import { ArrowDownToLine, ArrowUpFromLine } from "lucide-react"

import { formatDate } from "@lana/web/utils"
import { Button } from "@lana/web/ui/button"

import { LiquidationStatusBadge } from "../status-badge"

import RecordCollateralSentDialog from "./record-collateral-sent-dialog"
import RecordPaymentReceivedDialog from "./record-payment-received-dialog"

import Balance from "@/components/balance/balance"
import { DetailsCard, DetailItemProps } from "@/components/details"

import { GetLiquidationDetailsQuery } from "@/lib/graphql/generated"

type LiquidationDetailsProps = {
  liquidation: NonNullable<GetLiquidationDetailsQuery["liquidation"]>
}

export const LiquidationDetailsCard: React.FC<LiquidationDetailsProps> = ({
  liquidation,
}) => {
  const [openCollateralSentDialog, setOpenCollateralSentDialog] = useState(false)
  const [openPaymentReceivedDialog, setOpenPaymentReceivedDialog] = useState(false)
  const t = useTranslations("Liquidations.LiquidationDetails.DetailsCard")

  const details: DetailItemProps[] = [
    {
      label: t("details.customerEmail"),
      value: liquidation.creditFacility.customer.email,
      href: `/customers/${liquidation.creditFacility.customer.publicId}`,
    },
    {
      label: t("details.status"),
      value: <LiquidationStatusBadge completed={liquidation.completed} />,
    },
    {
      label: t("details.expectedToReceive"),
      value: <Balance amount={liquidation.expectedToReceive} currency="usd" />,
    },
    {
      label: t("details.createdAt"),
      value: formatDate(liquidation.createdAt),
    },
    {
      label: t("details.sentTotal"),
      value: <Balance amount={liquidation.sentTotal} currency="btc" />,
    },
    {
      label: t("details.receivedTotal"),
      value: <Balance amount={liquidation.receivedTotal} currency="usd" />,
    },
  ]

  const footerContent = (
    <>
      <Button
        variant="outline"
        onClick={() => setOpenCollateralSentDialog(true)}
        data-testid="record-collateral-sent-button"
      >
        <ArrowUpFromLine className="h-4 w-4 mr-2" />
        {t("buttons.recordCollateralSent")}
      </Button>
      <Button
        variant="outline"
        onClick={() => setOpenPaymentReceivedDialog(true)}
        data-testid="record-payment-received-button"
      >
        <ArrowDownToLine className="h-4 w-4 mr-2" />
        {t("buttons.recordPaymentReceived")}
      </Button>
    </>
  )

  return (
    <>
      <DetailsCard title={t("title")} details={details} footerContent={footerContent} />

      <RecordCollateralSentDialog
        open={openCollateralSentDialog}
        onOpenChange={setOpenCollateralSentDialog}
        liquidationId={liquidation.liquidationId}
      />
      <RecordPaymentReceivedDialog
        open={openPaymentReceivedDialog}
        onOpenChange={setOpenPaymentReceivedDialog}
        liquidationId={liquidation.liquidationId}
      />
    </>
  )
}
