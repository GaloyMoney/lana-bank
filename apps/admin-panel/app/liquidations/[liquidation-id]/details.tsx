"use client"

import React, { useState } from "react"
import { useTranslations } from "next-intl"
import { ArrowDownToLine, ArrowUpFromLine, Calculator } from "lucide-react"

import { formatDate } from "@lana/web/utils"
import { Button } from "@lana/web/ui/button"

import { LiquidationStatusBadge } from "../status-badge"

import RecordCollateralSentDialog from "./record-collateral-sent-dialog"
import RecordPaymentReceivedDialog from "./record-payment-received-dialog"
import LiquidationPaymentCalculatorDialog from "./liquidation-payment-calculator-dialog"

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
  const [openCalculatorDialog, setOpenCalculatorDialog] = useState(false)
  const t = useTranslations("Liquidations.LiquidationDetails.DetailsCard")

  const details: DetailItemProps[] = [
    {
      label: t("details.customerEmail"),
      value: liquidation.collateral.creditFacility?.customer.email,
      href: `/customers/${liquidation.collateral.creditFacility?.customer.publicId}`,
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
      label: t("details.amountReceived"),
      value: <Balance amount={liquidation.amountReceived} currency="usd" />,
    },
  ]

  const footerContent = (
    <>
      <Button
        variant="outline"
        onClick={() => setOpenCalculatorDialog(true)}
        data-testid="liquidation-payment-calculator-button"
      >
        <Calculator className="h-4 w-4 mr-2" />
        {t("buttons.calculatePayment")}
      </Button>
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

      <LiquidationPaymentCalculatorDialog
        open={openCalculatorDialog}
        onOpenChange={setOpenCalculatorDialog}
        liquidationId={liquidation.liquidationId}
        outstanding={liquidation.collateral.creditFacility?.balance?.outstanding?.usdBalance ?? 0}
        defaultToReceive={liquidation.expectedToReceive}
        defaultToLiquidate={liquidation.sentTotal}
      />
      <RecordCollateralSentDialog
        open={openCollateralSentDialog}
        onOpenChange={setOpenCollateralSentDialog}
        collateralId={liquidation.collateralId}
      />
      <RecordPaymentReceivedDialog
        open={openPaymentReceivedDialog}
        onOpenChange={setOpenPaymentReceivedDialog}
        collateralId={liquidation.collateralId}
      />
    </>
  )
}
