"use client"

import React from "react"
import Link from "next/link"

import { CreditFacilityCollateralUpdateDialog } from "../collateral-update"
import { CreditFacilityApproveDialog } from "../approve"
import { CreditFacilityDisbursementInitiateDialog } from "../disbursement-Initiate"
import { CreditFacilityCompleteDialog } from "../complete"
import { CreditFacilityPartialPaymentDialog } from "../partial-payment"

import {
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

type CreditFacilityDetailsProps = {
  creditFacilityId: string
  creditFacilityDetails: NonNullable<GetCreditFacilityDetailsQuery["creditFacility"]>
}

const CreditFacilityDetailsCard: React.FC<CreditFacilityDetailsProps> = ({
  creditFacilityId,
  creditFacilityDetails,
}) => {
  const [openCollateralUpdateDialog, setOpenCollateralUpdateDialog] =
    React.useState(false)
  const [openDisbursementInitiateDialog, setOpenDisbursementInitiateDialog] =
    React.useState(false)
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
          {creditFacilityDetails.userCanApprove &&
            [
              CreditFacilityStatus.PendingCollateralization,
              CreditFacilityStatus.PendingApproval,
            ].includes(creditFacilityDetails.status) &&
            creditFacilityDetails.collateral > 0 && (
              <Button
                variant="primary"
                className="w-full"
                onClick={() => setOpenApproveDialog(true)}
              >
                Approve
              </Button>
            )}
          {creditFacilityDetails.userCanUpdateCollateral && (
            <Button
              variant="primary"
              className="w-full"
              onClick={() => setOpenCollateralUpdateDialog(true)}
            >
              Update Collateral
            </Button>
          )}
          {creditFacilityDetails.userCanInitiateDisbursement &&
            creditFacilityDetails.status === CreditFacilityStatus.Active && (
              <Button
                variant="primary"
                className="w-full"
                onClick={() => setOpenDisbursementInitiateDialog(true)}
              >
                Initiate Disbursement
              </Button>
            )}
          {creditFacilityDetails.userCanComplete &&
            creditFacilityDetails.canBeCompleted && (
              <Button
                variant="primary"
                className="w-full"
                onClick={() => setOpenCompleteDialog(true)}
              >
                Complete Credit Facility
              </Button>
            )}
          {creditFacilityDetails.userCanRecordPayment &&
            creditFacilityDetails.status === CreditFacilityStatus.Active && (
              <Button
                variant="primary"
                className="w-full"
                onClick={() => setOpenPartialPaymentDialog(true)}
              >
                Make Payment
              </Button>
            )}
        </div>
      )}

      <CreditFacilityCollateralUpdateDialog
        creditFacilityId={creditFacilityId}
        openDialog={openCollateralUpdateDialog}
        setOpenDialog={setOpenCollateralUpdateDialog}
      />
      <CreditFacilityDisbursementInitiateDialog
        creditFacilityId={creditFacilityId}
        openDialog={openDisbursementInitiateDialog}
        setOpenDialog={setOpenDisbursementInitiateDialog}
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
    </div>
  )
}

export default CreditFacilityDetailsCard
