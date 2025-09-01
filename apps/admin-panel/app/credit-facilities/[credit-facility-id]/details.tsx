"use client"

import React from "react"
import { useTranslations } from "next-intl"

import { Button } from "@lana/web/ui/button"

import { formatDate } from "@lana/web/utils"

import { toast } from "sonner"

import { ExternalLinkIcon, CopyIcon } from "lucide-react"

import { Label } from "@lana/web/ui/label"

import { CreditFacilityCollateralUpdateDialog } from "../collateral-update"

import { CollateralizationStateLabel } from "../label"

import { CreditFacilityTermsDialog } from "./terms-dialog"

import {
  ApprovalProcessStatus,
  CreditFacilityRepaymentType,
  GetCreditFacilityLayoutDetailsQuery,
  WalletNetwork,
} from "@/lib/graphql/generated"
import { LoanAndCreditFacilityStatusBadge } from "@/app/credit-facilities/status-badge"
import ApprovalDialog from "@/app/actions/approve"
import DenialDialog from "@/app/actions/deny"
import { DetailsCard, DetailItemProps } from "@/components/details"
import { removeUnderscore } from "@/lib/utils"
import Balance from "@/components/balance/balance"
import { useLoanAgreement } from "@/hooks/use-loan-agreement"

type CreditFacilityDetailsProps = {
  creditFacilityId: string
  creditFacilityDetails: NonNullable<
    GetCreditFacilityLayoutDetailsQuery["creditFacilityByPublicId"]
  >
}

const CreditFacilityDetailsCard: React.FC<CreditFacilityDetailsProps> = ({
  creditFacilityId,
  creditFacilityDetails,
}) => {
  const t = useTranslations("CreditFacilities.CreditFacilityDetails.DetailsCard")
  const commonT = useTranslations("Common")

  const [openCollateralUpdateDialog, setOpenCollateralUpdateDialog] =
    React.useState(false)
  const [openApprovalDialog, setOpenApprovalDialog] = React.useState(false)
  const [openDenialDialog, setOpenDenialDialog] = React.useState(false)
  const [openTermsDialog, setOpenTermsDialog] = React.useState(false)

  const { generateLoanAgreementPdf, isGenerating } = useLoanAgreement()

  const handleGenerateLoanAgreement = () => {
    generateLoanAgreementPdf(creditFacilityDetails.customer.customerId)
  }

  const monthlyPaymentAmount = creditFacilityDetails.repaymentPlan.find(
    (plan) => plan.repaymentType === CreditFacilityRepaymentType.Interest,
  )?.initial

  const details: DetailItemProps[] = [
    {
      label: t("details.customer"),
      value: `${creditFacilityDetails.customer.email} (${removeUnderscore(creditFacilityDetails.customer.customerType)})`,
      href: `/customers/${creditFacilityDetails.customer.publicId}`,
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
      label: t("details.status"),
      value: (
        <LoanAndCreditFacilityStatusBadge
          data-testid="credit-facility-status-badge"
          status={creditFacilityDetails.status}
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
      value: formatDate(creditFacilityDetails.createdAt),
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
        <div className="flex items-center">
          <span
            className="font-mono text-sm truncate max-w-[200px]"
            title={creditFacilityDetails.wallet.address}
          >
            {creditFacilityDetails.wallet.address}
          </span>
          <CopyIcon
            onClick={() => {
              navigator.clipboard.writeText(creditFacilityDetails.wallet!.address)
              toast.success(commonT("copiedToClipboard"))
            }}
            className="h-3 w-3 shrink-0 cursor-pointer hover:text-blue-500"
            aria-hidden="true"
          />
        </div>
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
        {t("buttons.loanTerms")}
      </Button>
      <Button
        variant="outline"
        onClick={handleGenerateLoanAgreement}
        loading={isGenerating}
        data-testid="loan-agreement-button"
      >
        {t("buttons.loanAgreement")}
      </Button>
      {creditFacilityDetails.userCanUpdateCollateral && (
        <Button
          variant="outline"
          data-testid="update-collateral-button"
          onClick={() => setOpenCollateralUpdateDialog(true)}
        >
          {t("buttons.updateCollateral")}
        </Button>
      )}
      {creditFacilityDetails.approvalProcess.status ===
        ApprovalProcessStatus.InProgress &&
        creditFacilityDetails.approvalProcess.userCanSubmitDecision && (
          <>
            <Button
              data-testid="credit-facility-approve-button"
              variant="outline"
              onClick={() => setOpenApprovalDialog(true)}
            >
              {t("buttons.approve")}
            </Button>
            <Button
              data-testid="credit-facility-deny-button"
              variant="outline"
              onClick={() => setOpenDenialDialog(true)}
            >
              {t("buttons.deny")}
            </Button>
          </>
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
        errorMessage={creditFacilityDetails.approvalProcess.deniedReason}
      />

      <CreditFacilityTermsDialog
        creditFacility={creditFacilityDetails}
        openTermsDialog={openTermsDialog}
        setOpenTermsDialog={setOpenTermsDialog}
      />
      <CreditFacilityCollateralUpdateDialog
        creditFacilityId={creditFacilityId}
        currentCollateral={creditFacilityDetails.balance.collateral.btcBalance}
        collateralToMatchInitialCvl={creditFacilityDetails.collateralToMatchInitialCvl}
        openDialog={openCollateralUpdateDialog}
        setOpenDialog={setOpenCollateralUpdateDialog}
      />
      <ApprovalDialog
        approvalProcess={creditFacilityDetails?.approvalProcess}
        openApprovalDialog={openApprovalDialog}
        setOpenApprovalDialog={() => {
          setOpenApprovalDialog(false)
        }}
      />
      <DenialDialog
        approvalProcess={creditFacilityDetails?.approvalProcess}
        openDenialDialog={openDenialDialog}
        setOpenDenialDialog={() => {
          setOpenDenialDialog(false)
        }}
      />
    </>
  )
}

export default CreditFacilityDetailsCard

const MEMPOOL_BASE = {
  MAINNET: "https://mempool.space/address",
  TESTNET_3: "https://mempool.space/testnet/address",
  TESTNET_4: "https://mempool.space/testnet4/address",
} satisfies Record<WalletNetwork, string>

export function mempoolAddressUrl(address: string, network: WalletNetwork) {
  return `${MEMPOOL_BASE[network]}/${encodeURIComponent(address)}`
}
