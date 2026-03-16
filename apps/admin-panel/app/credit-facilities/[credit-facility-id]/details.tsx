"use client"

import React from "react"
import { useTranslations } from "next-intl"

import { Button } from "@lana/web/ui/button"

import { formatDate } from "@lana/web/utils"

import { toast } from "sonner"

import { ExternalLinkIcon, FileText, Download, RefreshCw, CheckCircle } from "lucide-react"

import { Label } from "@lana/web/ui/label"

import { CreditFacilityCollateralUpdateDialog } from "../collateral-update"

import { CollateralizationStateLabel } from "../label"

import { CreditFacilityTermsDialog } from "./terms-dialog"
import { CompleteCreditFacilityDialog } from "./complete-credit-facility"

import {
  GetCreditFacilityLayoutDetailsQuery,
  CreditFacilityStatus,
  WalletNetwork,
} from "@/lib/graphql/generated"
import { LoanAndCreditFacilityStatusBadge } from "@/app/credit-facilities/status-badge"
import { DetailsCard, DetailItemProps } from "@/components/details"
import { CustomerLabel } from "@/app/customers/customer-label"
import { useLoanAgreement } from "@/hooks/use-loan-agreement"
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
  const [openCompleteDialog, setOpenCompleteDialog] = React.useState(false)

  const { generateLoanAgreementPdf, isGenerating } = useLoanAgreement()

  const handleGenerateLoanAgreement = () => {
    generateLoanAgreementPdf(creditFacilityDetails.customer.customerId)
  }

  const details: DetailItemProps[] = [
    {
      label: t("details.customer"),
      value: (
        <CustomerLabel
          email={creditFacilityDetails.customer.email}
          customerType={creditFacilityDetails.customer.customerType}
        />
      ),
      href: `/customers/${creditFacilityDetails.customer.publicId}`,
    },
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
      value: creditFacilityDetails.wallet.custodian.name,
    },
    creditFacilityDetails.wallet.address && {
      label: (
        <Label className="inline-flex items-center">
          {t("details.walletAddress")}
          <a
            href={mempoolAddressUrl(
              creditFacilityDetails.wallet.address,
              creditFacilityDetails.wallet.network,
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
            navigator.clipboard.writeText(creditFacilityDetails.wallet.address)
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
      <Button
        variant="outline"
        onClick={handleGenerateLoanAgreement}
        loading={isGenerating}
        data-testid="loan-agreement-button"
      >
        <Download className="h-4 w-4 mr-2" />
        {t("buttons.loanAgreement")}
      </Button>
      {creditFacilityDetails.userCanUpdateCollateral && creditFacilityDetails.wallet.custodian.provider === "manual" && (
        <Button
          variant="outline"
          data-testid="update-collateral-button"
          onClick={() => setOpenCollateralUpdateDialog(true)}
        >
          <RefreshCw className="h-4 w-4 mr-2" />
          {t("buttons.updateCollateral")}
        </Button>
      )}
      {creditFacilityDetails.userCanComplete &&
        creditFacilityDetails.status === CreditFacilityStatus.Active && (
          <Button
            variant="destructive"
            data-testid="complete-credit-facility-button"
            onClick={() => setOpenCompleteDialog(true)}
          >
            <CheckCircle className="h-4 w-4 mr-2" />
            {t("buttons.complete")}
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
      <CompleteCreditFacilityDialog
        creditFacilityId={creditFacilityDetails.creditFacilityId}
        openCompleteDialog={openCompleteDialog}
        setOpenCompleteDialog={setOpenCompleteDialog}
      />
    </>
  )
}

export default CreditFacilityDetailsCard

const MEMPOOL_BASE = {
  MAINNET: "https://mempool.space/address",
  SIGNET: "https://mempool.space/signet/address",
  TESTNET3: "https://mempool.space/testnet/address",
  TESTNET4: "https://mempool.space/testnet4/address",
} satisfies Record<WalletNetwork, string>

export function mempoolAddressUrl(address: string, network: WalletNetwork) {
  return `${MEMPOOL_BASE[network]}/${encodeURIComponent(address)}`
}
