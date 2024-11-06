"use client"

import React from "react"
import Link from "next/link"

import { CreditFacilityCollateralUpdateDialog } from "../collateral-update"
import { CreditFacilityApproveDialog } from "../approve"
import { CreditFacilityDisbursalInitiateDialog } from "../disbursal-initiate"
import { CreditFacilityCompleteDialog } from "../complete"
import { CreditFacilityPartialPaymentDialog } from "../partial-payment"

import {
  ApprovalProcess,
  ApprovalProcessStatus,
  CreditFacility,
  CreditFacilityStatus,
  GetCreditFacilityDetailsQuery,
} from "@/lib/graphql/generated"
import { DetailItem, DetailsGroup } from "@/components/details"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/primitive/card"
import Balance from "@/components/balance/balance"
import { formatCollateralizationState } from "@/lib/utils"

import { Button } from "@/components/primitive/button"
import { LoanAndCreditFacilityStatusBadge } from "@/app/loans/status-badge"
import ApprovalDialog from "@/app/approval-process/approve"
import DenialDialog from "@/app/approval-process/deny"

type CreditFacilityDetailsProps = {
  creditFacilityId: string
  creditFacilityDetails: NonNullable<GetCreditFacilityDetailsQuery["creditFacility"]>
  refetch: () => void
}

const CreditFacilityDetailsCard: React.FC<CreditFacilityDetailsProps> = ({
  creditFacilityId,
  creditFacilityDetails,
  refetch,
}) => {
  const [openCollateralUpdateDialog, setOpenCollateralUpdateDialog] =
    React.useState(false)
  const [openDisbursalInitiateDialog, setOpenDisbursalInitiateDialog] =
    React.useState(false)
  const [openApprovalDialog, setOpenApprovalDialog] = React.useState(false)
  const [openDenialDialog, setOpenDenialDialog] = React.useState(false)
  const [openApproveDialog, setOpenApproveDialog] = React.useState(false)
  const [openCompleteDialog, setOpenCompleteDialog] = React.useState(false)
  const [openPartialPaymentDialog, setOpenPartialPaymentDialog] = React.useState(false)

  return (
    <div className="flex">
      <Card className="w-full">
        <>
          <CardHeader className="flex-row justify-between items-center">
            <CardTitle>Credit Facility</CardTitle>
            <LoanAndCreditFacilityStatusBadge status={creditFacilityDetails.status} />
          </CardHeader>
          <CardContent>
            <DetailsGroup>
              <DetailItem
                label="Credit Facility ID"
                value={creditFacilityDetails.creditFacilityId}
              />
              <Link href={`/customers/${creditFacilityDetails.customer.customerId}`}>
                <DetailItem
                  hover={true}
                  label="Customer Email"
                  value={creditFacilityDetails.customer.email}
                />
              </Link>
              <DetailItem
                label="Facility Amount"
                value={
                  <Balance amount={creditFacilityDetails.facilityAmount} currency="usd" />
                }
              />
              <DetailItem
                label="Collateralization State"
                value={formatCollateralizationState(
                  creditFacilityDetails.collateralizationState,
                )}
              />
            </DetailsGroup>
          </CardContent>
        </>
      </Card>
      {creditFacilityDetails.status !== CreditFacilityStatus.Closed && (
        <div className="flex flex-col space-y-2 mt-1 ml-4">
          {creditFacilityDetails.subjectCanUpdateCollateral && (
            <Button
              variant="primary"
              className="w-full"
              onClick={() => setOpenCollateralUpdateDialog(true)}
            >
              Update Collateral
            </Button>
          )}
          {creditFacilityDetails.subjectCanInitiateDisbursal &&
            creditFacilityDetails.status === CreditFacilityStatus.Active && (
              <Button
                variant="primary"
                className="w-full"
                onClick={() => setOpenDisbursalInitiateDialog(true)}
              >
                Initiate Disbursal
              </Button>
            )}
          {creditFacilityDetails.subjectCanComplete &&
            creditFacilityDetails.canBeCompleted && (
              <Button
                variant="primary"
                className="w-full"
                onClick={() => setOpenCompleteDialog(true)}
              >
                Complete Credit Facility
              </Button>
            )}
          {creditFacilityDetails.subjectCanRecordPayment &&
            creditFacilityDetails.status === CreditFacilityStatus.Active && (
              <Button
                variant="primary"
                className="w-full"
                onClick={() => setOpenPartialPaymentDialog(true)}
              >
                Make Payment
              </Button>
            )}
          {creditFacilityDetails.approvalProcess.status ===
            ApprovalProcessStatus.InProgress &&
            creditFacilityDetails.approvalProcess.subjectCanSubmitDecision && (
              <>
                <Button onClick={() => setOpenApprovalDialog(true)} className="ml-2">
                  Approve
                </Button>
                <Button onClick={() => setOpenDenialDialog(true)} className="ml-2">
                  Deny
                </Button>
              </>
            )}
        </div>
      )}

      <CreditFacilityCollateralUpdateDialog
        creditFacilityId={creditFacilityId}
        openDialog={openCollateralUpdateDialog}
        setOpenDialog={setOpenCollateralUpdateDialog}
      />
      <CreditFacilityDisbursalInitiateDialog
        creditFacilityId={creditFacilityId}
        openDialog={openDisbursalInitiateDialog}
        setOpenDialog={setOpenDisbursalInitiateDialog}
      />
      <CreditFacilityApproveDialog
        creditFacilityDetails={creditFacilityDetails as CreditFacility}
        openDialog={openApproveDialog}
        setOpenDialog={setOpenApproveDialog}
      />
      <CreditFacilityCompleteDialog
        creditFacilityId={creditFacilityId}
        openDialog={openCompleteDialog}
        setOpenDialog={setOpenCompleteDialog}
      />
      <CreditFacilityPartialPaymentDialog
        creditFacilityId={creditFacilityId}
        openDialog={openPartialPaymentDialog}
        setOpenDialog={setOpenPartialPaymentDialog}
      />
      <ApprovalDialog
        approvalProcess={creditFacilityDetails?.approvalProcess as ApprovalProcess}
        openApprovalDialog={openApprovalDialog}
        setOpenApprovalDialog={() => {
          setOpenApprovalDialog(false)
        }}
        refetch={refetch}
      />
      <DenialDialog
        approvalProcess={creditFacilityDetails?.approvalProcess as ApprovalProcess}
        openDenialDialog={openDenialDialog}
        setOpenDenialDialog={() => {
          setOpenDenialDialog(false)
        }}
        refetch={refetch}
      />
    </div>
  )
}

export default CreditFacilityDetailsCard
