"use client"
import { useState } from "react"
import Link from "next/link"

import { DisbursalStatusBadge } from "../status-badge"

import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/primitive/card"
import { DetailItem, DetailsGroup } from "@/components/details"
import { Button } from "@/components/primitive/button"
import Balance from "@/components/balance/balance"
import {
  ApprovalProcess,
  ApprovalProcessStatus,
  GetDisbursalDetailsQuery,
} from "@/lib/graphql/generated"
import ApprovalDialog from "@/app/approval-process/approve"
import DenialDialog from "@/app/approval-process/deny"

type DisbursalDetailsProps = {
  disbursal: NonNullable<GetDisbursalDetailsQuery["disbursal"]>
  refetch: () => void
}

export const DisbursalDetailsCard: React.FC<DisbursalDetailsProps> = ({
  disbursal,
  refetch,
}) => {
  const [openApprovalDialog, setOpenApprovalDialog] = useState(false)
  const [openDenialDialog, setOpenDenialDialog] = useState(false)

  return (
    <div className="flex">
      <Card className="w-full">
        <CardHeader className="flex flex-row justify-between items-center">
          <CardTitle>Disbursal</CardTitle>
          <DisbursalStatusBadge status={disbursal.status} />
        </CardHeader>
        <CardContent>
          <DetailsGroup>
            <Link href={`/customers/${disbursal.creditFacility.customer.customerId}`}>
              <DetailItem
                hover={true}
                label="Customer Email"
                value={disbursal.creditFacility.customer.email}
              />
            </Link>

            <DetailItem
              label="Disbursal Amount"
              value={<Balance amount={disbursal.amount} currency="usd" />}
            />

            <DetailItem
              label="Facility Amount"
              value={
                <Balance
                  amount={disbursal.creditFacility.facilityAmount}
                  currency="usd"
                />
              }
            />
          </DetailsGroup>
        </CardContent>
        {disbursal.approvalProcess.deniedReason && (
          <CardFooter className="text-destructive">
            Denied Reason: {disbursal.approvalProcess.deniedReason}
          </CardFooter>
        )}
      </Card>
      <div className="flex flex-col space-y-2 mt-1 ml-4">
        {disbursal.approvalProcess?.status === ApprovalProcessStatus.InProgress &&
          disbursal.approvalProcess.subjectCanSubmitDecision && (
            <>
              <Button onClick={() => setOpenApprovalDialog(true)} variant="outline">
                Approve
              </Button>
              <Button onClick={() => setOpenDenialDialog(true)} variant="outline">
                Deny
              </Button>
            </>
          )}
        <Link href={`/credit-facilities/${disbursal.creditFacility.creditFacilityId}`}>
          <Button variant="outline">View Credit Facility</Button>
        </Link>
      </div>
      <ApprovalDialog
        approvalProcess={disbursal.approvalProcess as ApprovalProcess}
        openApprovalDialog={openApprovalDialog}
        setOpenApprovalDialog={() => setOpenApprovalDialog(false)}
        refetch={refetch}
      />
      <DenialDialog
        approvalProcess={disbursal.approvalProcess as ApprovalProcess}
        openDenialDialog={openDenialDialog}
        setOpenDenialDialog={() => setOpenDenialDialog(false)}
        refetch={refetch}
      />
    </div>
  )
}
