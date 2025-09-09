"use client"

import React from "react"
import { useTranslations } from "next-intl"
import { Button } from "@lana/web/ui/button"
import { formatDate } from "@lana/web/utils"
import Link from "next/link"
import { RefreshCw, Check, X, ArrowRight } from "lucide-react"

import { DetailsCard, DetailItemProps } from "@/components/details"
import Balance from "@/components/balance/balance"
import {
  CreditFacilityProposal,
  ApprovalProcessStatus,
  ApprovalProcessFieldsFragment,
} from "@/lib/graphql/generated"
import { usePublicIdForCreditFacility } from "@/hooks/use-public-id"
import ApprovalDialog from "@/app/actions/approve"
import DenialDialog from "@/app/actions/deny"
import { VotersCard } from "@/app/disbursals/[disbursal-id]/voters"

import { CreditFacilityProposalStatusBadge } from "../status-badge"
import { CreditFacilityProposalCollateralizationStateLabel } from "../label"
import { CreditFacilityProposalCollateralUpdateDialog } from "../collateral-update"
import { Alert, AlertDescription, AlertTitle } from "@lana/web/ui/alert"
import { removeUnderscore } from "@/lib/utils"
type CreditFacilityProposalDetailsCardProps = {
  proposalDetails: CreditFacilityProposal
  approvalProcess?: ApprovalProcessFieldsFragment | null
}

const CreditFacilityProposalDetailsCard: React.FC<
  CreditFacilityProposalDetailsCardProps
> = ({ proposalDetails, approvalProcess }) => {
  const t = useTranslations("CreditFacilityProposals.ProposalDetails.DetailsCard")

  const [openCollateralUpdateDialog, setOpenCollateralUpdateDialog] =
    React.useState(false)
  const [openApprovalDialog, setOpenApprovalDialog] = React.useState(false)
  const [openDenialDialog, setOpenDenialDialog] = React.useState(false)

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
  ]

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
      {approvalProcess?.status === ApprovalProcessStatus.InProgress &&
        approvalProcess.userCanSubmitDecision && (
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
        errorMessage={approvalProcess?.deniedReason ?? undefined}
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
      {approvalProcess && <VotersCard approvalProcess={approvalProcess} />}
      <ApprovalDialog
        approvalProcess={approvalProcess as ApprovalProcessFieldsFragment}
        openApprovalDialog={openApprovalDialog}
        setOpenApprovalDialog={() => setOpenApprovalDialog(false)}
      />
      <DenialDialog
        approvalProcess={approvalProcess as ApprovalProcessFieldsFragment}
        openDenialDialog={openDenialDialog}
        setOpenDenialDialog={() => setOpenDenialDialog(false)}
      />
    </>
  )
}

export default CreditFacilityProposalDetailsCard
