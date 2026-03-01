"use client"

import React from "react"
import { useTranslations } from "next-intl"

import { Button } from "@lana/web/ui/button"

import { formatDate } from "@lana/web/utils"

import { toast } from "sonner"

import { ExternalLinkIcon, FileText, RefreshCw } from "lucide-react"

import { Label } from "@lana/web/ui/label"

import { CreditFacilityCollateralUpdateDialog } from "../collateral-update"

import { CollateralizationStateLabel } from "../label"

import { CreditFacilityTermsDialog } from "./terms-dialog"

import {
  CreditFacilityRepaymentType,
  GetCreditFacilityLayoutDetailsQuery,
  WalletNetwork,
} from "@/lib/graphql/generated"
import { LoanAndCreditFacilityStatusBadge } from "@/app/credit-facilities/status-badge"
import { DetailsCard, DetailItemProps } from "@/components/details"
import Balance from "@/components/balance/balance"

type CreditFacilityDetailsProps = {
  creditFacilityDetails: NonNullable<
    GetCreditFacilityLayoutDetailsQuery["creditFacilityByPublicId"]
  >
}

const CreditFacilityDetailsCard: React.FC<CreditFacilityDetailsProps> = ({
  creditFacilityDetails,
}) => {
  const t = useTranslations("CreditFacilities.CreditFacilityDetails.DetailsCard")
  const commonT = useTranslations("Common")

  const [openCollateralUpdateDialog, setOpenCollateralUpdateDialog] =
    React.useState(false)
  const [openTermsDialog, setOpenTermsDialog] = React.useState(false)

  const monthlyPaymentAmount = creditFacilityDetails.repaymentPlan.find(
    (plan) => plan.repaymentType === CreditFacilityRepaymentType.Interest,
  )?.initial

  const details: DetailItemProps[] = [
    {
      label: t("details.status"),
      value: (
        <LoanAndCreditFacilityStatusBadge
          data-testid="credit-facility-status-badge"
          status={creditFacilityDetails.status}
        />
      ),
    },
    {
      label: t("details.collateralizationState"),
      value: (
        <CollateralizationStateLabel
          state={creditFacilityDetails.collateralizationState}
        />
      ),
    },
    {
      label: t("details.monthlyPayment"),
      value: monthlyPaymentAmount ? (
        <Balance amount={monthlyPaymentAmount} currency="usd" />
      ) : (
        t("details.noMonthlyPaymentAvailable")
      ),
    },
    {
      label: t("details.dateOfIssuance"),
      value: formatDate(creditFacilityDetails.activatedAt),
    },
    {
      label: t("details.maturityDate"),
      value: formatDate(creditFacilityDetails.maturesAt),
      displayCondition: creditFacilityDetails.maturesAt !== null,
    },
    {
      label: t("details.custodian"),
      value: creditFacilityDetails.wallet?.custodian.name ?? t("details.manual"),
    },
    creditFacilityDetails.wallet?.address && {
      label: (
        <Label className="inline-flex items-center">
          {t("details.walletAddress")}
          <a
            href={mempoolAddressUrl(
              creditFacilityDetails.wallet!.address,
              creditFacilityDetails.wallet!.network,
            )}
            target="_blank"
            className="ml-2 inline-flex items-center gap-1 text-xs text-blue-500 whitespace-nowrap leading-none"
            onClick={(e) => e.stopPropagation()}
          >
            <span className="leading-none">{t("details.viewOnMempool")}</span>
            <ExternalLinkIcon className="h-2.5 w-2.5 shrink-0" aria-hidden="true" />
          </a>
        </Label>
      ),
      value: (
        <span
          onClick={() => {
            navigator.clipboard.writeText(creditFacilityDetails.wallet!.address)
            toast.success(commonT("copiedToClipboard"))
          }}
          className="cursor-pointer hover:bg-secondary font-mono text-sm"
          title={creditFacilityDetails.wallet.address}
        >
          {creditFacilityDetails.wallet.address}
        </span>
      ),
    },
  ].filter(Boolean) as DetailItemProps[]

  const footerContent = (
    <>
      <Button
        variant="outline"
        onClick={() => setOpenTermsDialog(true)}
        data-testid="loan-terms-button"
      >
        <FileText className="h-4 w-4 mr-2" />
        {t("buttons.loanTerms")}
      </Button>
      {creditFacilityDetails.userCanUpdateCollateral && (
        <Button
          variant="outline"
          data-testid="update-collateral-button"
          onClick={() => setOpenCollateralUpdateDialog(true)}
        >
          <RefreshCw className="h-4 w-4 mr-2" />
          {t("buttons.updateCollateral")}
        </Button>
      )}
    </>
  )

  return (
    <>
      <DetailsCard
        title={t("title")}
        details={details}
        columns={3}
        footerContent={footerContent}
      />

      <CreditFacilityTermsDialog
        creditFacility={creditFacilityDetails}
        openTermsDialog={openTermsDialog}
        setOpenTermsDialog={setOpenTermsDialog}
      />
      <CreditFacilityCollateralUpdateDialog
        collateralId={creditFacilityDetails.collateralId}
        currentCollateral={creditFacilityDetails.balance.collateral.btcBalance}
        collateralToMatchInitialCvl={creditFacilityDetails.collateralToMatchInitialCvl}
        openDialog={openCollateralUpdateDialog}
        setOpenDialog={setOpenCollateralUpdateDialog}
      />
    </>
  )
}

export default CreditFacilityDetailsCard

const MEMPOOL_BASE = {
  MAINNET: "https://mempool.space/address",
  TESTNET3: "https://mempool.space/testnet/address",
  TESTNET4: "https://mempool.space/testnet4/address",
} satisfies Record<WalletNetwork, string>

export function mempoolAddressUrl(address: string, network: WalletNetwork) {
  return `${MEMPOOL_BASE[network]}/${encodeURIComponent(address)}`
}
