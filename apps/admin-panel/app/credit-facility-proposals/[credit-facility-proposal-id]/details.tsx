"use client"

import React from "react"
import { useTranslations } from "next-intl"
import { Button } from "@lana/web/ui/button"
import { formatDate } from "@lana/web/utils"
import { Check, X, ArrowRight } from "lucide-react"
import Link from "next/link"

import { CreditFacilityProposalStatusBadge } from "../status-badge"

import { CustomerApprovalDialog } from "./customer-approval-dialog"

import { DetailsCard, DetailItemProps } from "@/components/details"
import Balance from "@/components/balance/balance"
import {
  ApprovalProcessStatus,
  ApprovalProcessFieldsFragment,
  GetCreditFacilityProposalLayoutDetailsQuery,
  CreditFacilityProposalStatus,
} from "@/lib/graphql/generated"
import ApprovalDialog from "@/app/actions/approve"
import DenialDialog from "@/app/actions/deny"
import { VotersCard } from "@/app/disbursals/[disbursal-id]/voters"

type CreditFacilityProposalDetailsCardProps = {
  proposalDetails: NonNullable<
    GetCreditFacilityProposalLayoutDetailsQuery["creditFacilityProposal"]
  >
}

const CreditFacilityProposalDetailsCard: React.FC<
  CreditFacilityProposalDetailsCardProps
> = ({ proposalDetails }) => {
  const t = useTranslations("CreditFacilityProposals.ProposalDetails.DetailsCard")

  const [openApprovalDialog, setOpenApprovalDialog] = React.useState(false)
  const [openDenialDialog, setOpenDenialDialog] = React.useState(false)
  const [openCustomerApprovalDialog, setOpenCustomerApprovalDialog] =
    React.useState(false)
  const [openCustomerDenialDialog, setOpenCustomerDenialDialog] = React.useState(false)

  const details: DetailItemProps[] = [
    {
      label: t("details.customer"),
      value: "-",
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
      label: t("details.facilityAmount"),
      value: <Balance amount={proposalDetails.facilityAmount} currency="usd" />,
    },
    {
      label: t("details.custodian"),
      value: proposalDetails.custodian?.name ?? t("details.manual"),
    },
    {
      label: t("details.createdAt"),
      value: formatDate(proposalDetails.createdAt),
    },
  ].filter(Boolean) as DetailItemProps[]

  const footerContent = (
    <>
      {proposalDetails.status ===
        CreditFacilityProposalStatus.PendingCustomerApproval && (
        <>
          <Button
            variant="outline"
            onClick={() => setOpenCustomerApprovalDialog(true)}
            data-testid="customer-approval-approve-button"
          >
            <Check className="h-4 w-4 mr-2" />
            {t("buttons.customerApprove")}
          </Button>
          <Button
            variant="outline"
            onClick={() => setOpenCustomerDenialDialog(true)}
            data-testid="customer-approval-deny-button"
          >
            <X className="h-4 w-4 mr-2" />
            {t("buttons.customerDeny")}
          </Button>
        </>
      )}
      {proposalDetails?.approvalProcess?.status === ApprovalProcessStatus.InProgress &&
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
      {proposalDetails.status === CreditFacilityProposalStatus.Approved && (
        <Button variant="outline" data-testid="view-pending-facility-button" asChild>
          <Link
            href={`/pending-credit-facilities/${proposalDetails.creditFacilityProposalId}`}
          >
            {t("buttons.viewPendingFacility")}
            <ArrowRight className="h-4 w-4 ml-2" />
          </Link>
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
        errorMessage={proposalDetails?.approvalProcess?.deniedReason ?? undefined}
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
      <CustomerApprovalDialog
        open={openCustomerApprovalDialog}
        onOpenChange={setOpenCustomerApprovalDialog}
        creditFacilityProposalId={proposalDetails.creditFacilityProposalId}
        approved={true}
        facilityAmount={proposalDetails.facilityAmount}
        customerEmail="-"
        createdAt={proposalDetails.createdAt}
      />
      <CustomerApprovalDialog
        open={openCustomerDenialDialog}
        onOpenChange={setOpenCustomerDenialDialog}
        creditFacilityProposalId={proposalDetails.creditFacilityProposalId}
        approved={false}
        facilityAmount={proposalDetails.facilityAmount}
        customerEmail="-"
        createdAt={proposalDetails.createdAt}
      />
    </>
  )
}

export default CreditFacilityProposalDetailsCard
