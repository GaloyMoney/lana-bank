"use client"

import React, { useState } from "react"
import { useTranslations } from "next-intl"
import { ArrowDownToLine, ArrowUpFromLine, Calculator, ChevronDown, ChevronUp } from "lucide-react"

import { formatDate } from "@lana/web/utils"
import { Button } from "@lana/web/ui/button"

import { LiquidationStatusBadge } from "../status-badge"

import RecordCollateralSentDialog from "./record-collateral-sent-dialog"
import RecordPaymentReceivedDialog from "./record-payment-received-dialog"
import LiquidationPaymentCalculatorPanel from "./liquidation-payment-calculator-panel"

import Balance from "@/components/balance/balance"
import { DetailsCard, DetailItemProps } from "@/components/details"

import { GetLiquidationDetailsQuery } from "@/lib/graphql/generated"
import { UsdCents } from "@/types"

type LiquidationDetailsProps = {
  liquidation: NonNullable<GetLiquidationDetailsQuery["liquidation"]>
}

export const LiquidationDetailsCard: React.FC<LiquidationDetailsProps> = ({
  liquidation,
}) => {
  const [openCollateralSentDialog, setOpenCollateralSentDialog] = useState(false)
  const [openPaymentReceivedDialog, setOpenPaymentReceivedDialog] = useState(false)
  const [calculatorOpen, setCalculatorOpen] = useState(false)
  const t = useTranslations("Liquidations.LiquidationDetails.DetailsCard")
  const calcT = useTranslations("Liquidations.LiquidationDetails.calculator")

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
      label: t("details.initiallyEstimatedToLiquidate"),
      value: <Balance amount={liquidation.initiallyEstimatedToLiquidate} currency="btc" />,
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

  const footerContent = !liquidation.completed ? (
    <div className="flex flex-col gap-4 w-full">
      <div className="flex flex-wrap gap-2 justify-end">
        <Button
          variant="outline"
          onClick={() => setCalculatorOpen(!calculatorOpen)}
          data-testid="liquidation-payment-calculator-button"
        >
          <Calculator className="h-4 w-4 mr-2" />
          {calcT("buttons.calculatePayment")}
          {calculatorOpen ? (
            <ChevronUp className="h-4 w-4 ml-2" />
          ) : (
            <ChevronDown className="h-4 w-4 ml-2" />
          )}
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
      </div>
      <LiquidationPaymentCalculatorPanel
        open={calculatorOpen}
        onClose={() => setCalculatorOpen(false)}
        liquidationId={liquidation.liquidationId}
        outstanding={liquidation.collateral.creditFacility?.balance?.outstanding?.usdBalance ?? 0 as UsdCents}
        defaultToReceive={liquidation.expectedToReceive}
        defaultToLiquidate={liquidation.initiallyEstimatedToLiquidate}
      />
    </div>
  ) : undefined

  return (
    <>
      <DetailsCard title={t("title")} details={details} footerContent={footerContent} />

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
