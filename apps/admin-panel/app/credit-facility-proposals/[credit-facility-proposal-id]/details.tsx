"use client"

import React from "react"
import { useTranslations } from "next-intl"
import { Button } from "@lana/web/ui/button"
import { formatDate } from "@lana/web/utils"
import Link from "next/link"
import { RefreshCw, Check, X, ArrowRight, ExternalLinkIcon } from "lucide-react"

import { Alert, AlertDescription, AlertTitle } from "@lana/web/ui/alert"

import { Label } from "@lana/web/ui/label"

import { toast } from "sonner"

import { CreditFacilityProposalStatusBadge } from "../status-badge"

import { CreditFacilityProposalCollateralizationStateLabel } from "../label"

import { CreditFacilityProposalCollateralUpdateDialog } from "../collateral-update"

import { DetailsCard, DetailItemProps } from "@/components/details"
import Balance from "@/components/balance/balance"
import {
  ApprovalProcessStatus,
  ApprovalProcessFieldsFragment,
  GetCreditFacilityProposalLayoutDetailsQuery,
} from "@/lib/graphql/generated"
import { usePublicIdForCreditFacility } from "@/hooks/use-public-id"
import ApprovalDialog from "@/app/actions/approve"
import DenialDialog from "@/app/actions/deny"
import { VotersCard } from "@/app/disbursals/[disbursal-id]/voters"

import { removeUnderscore } from "@/lib/utils"
import { mempoolAddressUrl } from "@/app/credit-facilities/[credit-facility-id]/details"

type CreditFacilityProposalDetailsCardProps = {
  proposalDetails: NonNullable<
    GetCreditFacilityProposalLayoutDetailsQuery["creditFacilityProposal"]
  >
}

const CreditFacilityProposalDetailsCard: React.FC<
  CreditFacilityProposalDetailsCardProps
> = ({ proposalDetails }) => {
  const t = useTranslations("CreditFacilityProposals.ProposalDetails.DetailsCard")

  const [openCollateralUpdateDialog, setOpenCollateralUpdateDialog] =
    React.useState(false)
  const [openApprovalDialog, setOpenApprovalDialog] = React.useState(false)
  const [openDenialDialog, setOpenDenialDialog] = React.useState(false)
  const commonT = useTranslations("Common")

  const details: DetailItemProps[] = [
    {
      label: t("details.customer"),
      value: `${proposalDetails.customer.email} (${removeUnderscore(proposalDetails.customer.customerType)})`,
      href: `/customers/${proposalDetails.customer.publicId}`,
    },
    {
      label: t("details.status"),
      value: (
        <CreditFacilityProposalStatusBadge
          status={proposalDetails.status}
          data-testid="proposal-status-badge"
        />
      ),
    },
    {
      label: t("details.collateralizationState"),
      value: (
        <CreditFacilityProposalCollateralizationStateLabel
          state={proposalDetails.collateralizationState}
        />
      ),
    },
    {
      label: t("details.facilityAmount"),
      value: <Balance amount={proposalDetails.facilityAmount} currency="usd" />,
    },
    {
      label: t("details.createdAt"),
      value: formatDate(proposalDetails.createdAt),
    },
    {
      label: t("details.custodian"),
      value: proposalDetails.wallet?.custodian.name ?? t("details.manual"),
    },
    proposalDetails.wallet?.address && {
      label: (
        <Label className="inline-flex items-center">
          {t("details.walletAddress")}
          <a
            href={mempoolAddressUrl(
              proposalDetails.wallet!.address,
              proposalDetails.wallet!.network,
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
            navigator.clipboard.writeText(proposalDetails.wallet!.address)
            toast.success(commonT("copiedToClipboard"))
          }}
          className="cursor-pointer hover:bg-secondary font-mono text-sm"
          title={proposalDetails.wallet.address}
        >
          {proposalDetails.wallet.address}
        </span>
      ),
    },
  ].filter(Boolean) as DetailItemProps[]
  const { publicId: facilityPublicId } = usePublicIdForCreditFacility(
    proposalDetails.creditFacilityProposalId,
  )

  const footerContent = (
    <>
      {proposalDetails.status !== "COMPLETED" && (
        <Button
          variant="outline"
          onClick={() => setOpenCollateralUpdateDialog(true)}
          data-testid="update-collateral-button"
        >
          <RefreshCw className="h-4 w-4 mr-2" />
          {t("buttons.updateCollateral")}
        </Button>
      )}
      {proposalDetails.approvalProcess.status === ApprovalProcessStatus.InProgress &&
        proposalDetails.approvalProcess.userCanSubmitDecision && (
          <>
            <Button
              variant="outline"
              onClick={() => setOpenApprovalDialog(true)}
              data-testid="approval-process-approve-button"
            >
              <Check className="h-4 w-4 mr-2" />
              {t("buttons.approve")}
            </Button>
            <Button
              variant="outline"
              onClick={() => setOpenDenialDialog(true)}
              data-testid="approval-process-deny-button"
            >
              <X className="h-4 w-4 mr-2" />
              {t("buttons.deny")}
            </Button>
          </>
        )}
      {proposalDetails.status === "COMPLETED" && facilityPublicId && (
        <Link href={`/credit-facilities/${facilityPublicId}`}>
          <Button variant="outline" data-testid="view-facility-button">
            {t("buttons.viewFacility")}
            <ArrowRight className="h-4 w-4 ml-2" />
          </Button>
        </Link>
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
        errorMessage={proposalDetails.approvalProcess.deniedReason ?? undefined}
      />

      {proposalDetails.status === "COMPLETED" && (
        <Alert className="mt-2 border-green-600/50 text-green-700 [&>svg]:text-green-700">
          <AlertTitle>{t("alerts.completedTitle")}</AlertTitle>
          <AlertDescription>
            {t("alerts.completedDescription")}{" "}
            <Link
              href={facilityPublicId ? `/credit-facilities/${facilityPublicId}` : "#"}
              className="underline"
            >
              {t("alerts.viewAssociatedFacility")}
            </Link>
          </AlertDescription>
        </Alert>
      )}

      <CreditFacilityProposalCollateralUpdateDialog
        openDialog={openCollateralUpdateDialog}
        setOpenDialog={setOpenCollateralUpdateDialog}
        creditFacilityProposalId={proposalDetails.creditFacilityProposalId}
        currentCollateral={proposalDetails.collateral.btcBalance}
        collateralToMatchInitialCvl={proposalDetails.collateralToMatchInitialCvl}
      />
      {proposalDetails.approvalProcess && (
        <VotersCard approvalProcess={proposalDetails.approvalProcess} />
      )}
      <ApprovalDialog
        approvalProcess={proposalDetails.approvalProcess as ApprovalProcessFieldsFragment}
        openApprovalDialog={openApprovalDialog}
        setOpenApprovalDialog={() => setOpenApprovalDialog(false)}
      />
      <DenialDialog
        approvalProcess={proposalDetails.approvalProcess as ApprovalProcessFieldsFragment}
        openDenialDialog={openDenialDialog}
        setOpenDenialDialog={() => setOpenDenialDialog(false)}
      />
    </>
  )
}

export default CreditFacilityProposalDetailsCard
