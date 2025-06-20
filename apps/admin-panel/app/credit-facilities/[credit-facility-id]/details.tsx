"use client"

import React from "react"
import { useTranslations } from "next-intl"

import { Button } from "@lana/web/ui/button"

import { formatDate } from "@lana/web/utils"

import { CreditFacilityCollateralUpdateDialog } from "../collateral-update"

import { CollateralizationStateLabel } from "../label"

import { CreditFacilityTermsDialog } from "./terms-dialog"

import {
  ApprovalProcessStatus,
  CreditFacilityRepaymentType,
  GetCreditFacilityLayoutDetailsQuery,
} from "@/lib/graphql/generated"
import { LoanAndCreditFacilityStatusBadge } from "@/app/credit-facilities/status-badge"
import ApprovalDialog from "@/app/actions/approve"
import DenialDialog from "@/app/actions/deny"
import { DetailsCard, DetailItemProps } from "@/components/details"
import { removeUnderscore } from "@/lib/utils"
import Balance from "@/components/balance/balance"

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
  const t = useTranslations("CreditFacilities.CreditFacilityDetails.DetailsCard")

  const [openCollateralUpdateDialog, setOpenCollateralUpdateDialog] =
    React.useState(false)
  const [openApprovalDialog, setOpenApprovalDialog] = React.useState(false)
  const [openDenialDialog, setOpenDenialDialog] = React.useState(false)
  const [openTermsDialog, setOpenTermsDialog] = React.useState(false)

  const monthlyPaymentAmount = creditFacilityDetails.repaymentPlan.find(
    (plan) => plan.repaymentType === CreditFacilityRepaymentType.Interest,
  )?.initial

  const details: DetailItemProps[] = [
    {
      label: t("details.customer"),
      value: `${creditFacilityDetails.customer.email} (${removeUnderscore(creditFacilityDetails.customer.customerType)})`,
      href: `/customers/${creditFacilityDetails.customer.customerId}`,
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
    },
  ]

  const footerContent = (
    <>
      <Button
        variant="outline"
        onClick={() => setOpenTermsDialog(true)}
        data-testid="loan-terms-button"
      >
        {t("buttons.loanTerms")}
      </Button>
      {creditFacilityDetails.subjectCanUpdateCollateral && (
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
        creditFacilityDetails.approvalProcess.subjectCanSubmitDecision && (
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
