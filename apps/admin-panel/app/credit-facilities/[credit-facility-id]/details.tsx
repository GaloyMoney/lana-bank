"use client"

import React from "react"

import { Button } from "@lana/web/ui/button"

import { CreditFacilityCollateralUpdateDialog } from "../collateral-update"

import { CreditFacilityTermsDialog } from "./terms-dialog"

import {
  ApprovalProcessStatus,
  GetCreditFacilityLayoutDetailsQuery,
} from "@/lib/graphql/generated"
import { formatCollateralizationState, formatDate } from "@/lib/utils"
import { LoanAndCreditFacilityStatusBadge } from "@/app/loans/status-badge"
import ApprovalDialog from "@/app/actions/approve"
import DenialDialog from "@/app/actions/deny"
import { DetailsCard, DetailItemProps } from "@/components/details"

type CreditFacilityDetailsProps = {
  creditFacilityId: string
  creditFacilityDetails: NonNullable<
    GetCreditFacilityLayoutDetailsQuery["creditFacility"]
  >
}

const CreditFacilityDetailsCard: React.FC<CreditFacilityDetailsProps> = ({
  creditFacilityId,
  creditFacilityDetails,
}) => {
  const [openCollateralUpdateDialog, setOpenCollateralUpdateDialog] =
    React.useState(false)
  const [openApprovalDialog, setOpenApprovalDialog] = React.useState(false)
  const [openDenialDialog, setOpenDenialDialog] = React.useState(false)
  const [openTermsDialog, setOpenTermsDialog] = React.useState(false)

  const details: DetailItemProps[] = [
    {
      label: "Customer Email",
      value: creditFacilityDetails.customer.email,
      href: `/customers/${creditFacilityDetails.customer.customerId}`,
    },
    {
      label: "Collateralization State",
      value: formatCollateralizationState(creditFacilityDetails.collateralizationState),
    },
    {
      label: "Status",
      value: (
        <LoanAndCreditFacilityStatusBadge
          data-testid="credit-facility-status-badge"
          status={creditFacilityDetails.status}
        />
      ),
    },
    {
      label: "Matures At",
      value: formatDate(creditFacilityDetails.maturesAt),
    },
  ]

  const footerContent = (
    <>
      <Button
        variant="outline"
        onClick={() => setOpenTermsDialog(true)}
        data-testid="loan-terms-button"
      >
        Loan Terms
      </Button>
      {creditFacilityDetails.subjectCanUpdateCollateral && (
        <Button
          variant="outline"
          data-testid="update-collateral-button"
          onClick={() => setOpenCollateralUpdateDialog(true)}
        >
          Update Collateral
        </Button>
      )}
      {creditFacilityDetails.approvalProcess.status ===
        ApprovalProcessStatus.InProgress &&
        creditFacilityDetails.approvalProcess.subjectCanSubmitDecision && (
          <>
            <Button
              data-testid="credit-facility-approve-button"
              variant="outline"
              onClick={() => setOpenApprovalDialog(true)}
            >
              Approve
            </Button>
            <Button
              data-testid="credit-facility-deny-button"
              variant="outline"
              onClick={() => setOpenDenialDialog(true)}
            >
              Deny
            </Button>
          </>
        )}
    </>
  )

  return (
    <>
      <DetailsCard
        title="Credit Facility"
        details={details}
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
