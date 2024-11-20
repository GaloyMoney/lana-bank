"use client"

import React, { useState } from "react"

import { WithdrawalStatusBadge } from "../status-badge"
import { WithdrawalConfirmDialog } from "../confirm"
import { WithdrawalCancelDialog } from "../cancel"

import DetailsCard, { DetailItemType } from "@/components/details-card"
import { Button } from "@/components/primitive/button"
import Balance from "@/components/balance/balance"
import {
  ApprovalProcess,
  ApprovalProcessStatus,
  GetWithdrawalDetailsQuery,
  WithdrawalStatus,
} from "@/lib/graphql/generated"
import ApprovalDialog from "@/app/approval-process/approve"
import DenialDialog from "@/app/approval-process/deny"
import { VotersCard } from "@/app/disbursals/[disbursal-id]/voters"

type WithdrawalDetailsProps = {
  withdrawal: NonNullable<GetWithdrawalDetailsQuery["withdrawal"]>
  refetch: () => void
}

const WithdrawalDetailsCard: React.FC<WithdrawalDetailsProps> = ({
  withdrawal,
  refetch,
}) => {
  const [openWithdrawalCancelDialog, setOpenWithdrawalCancelDialog] =
    useState<WithdrawalWithCustomer | null>(null)
  const [openWithdrawalConfirmDialog, setOpenWithdrawalConfirmDialog] =
    useState<WithdrawalWithCustomer | null>(null)
  const [openApprovalDialog, setOpenApprovalDialog] = useState(false)
  const [openDenialDialog, setOpenDenialDialog] = useState(false)

  const details: DetailItemType[] = [
    {
      label: "Customer Email",
      value: withdrawal.customer.email,
      href: `/customers/${withdrawal.customer.customerId}`,
    },
    {
      label: "Withdrawal Amount",
      value: <Balance amount={withdrawal.amount} currency="usd" />,
    },
    {
      label: "Withdrawal Reference",
      value:
        withdrawal.reference === withdrawal.withdrawalId ? "n/a" : withdrawal.reference,
    },
    {
      label: "Status",
      value: <WithdrawalStatusBadge status={withdrawal.status} />,
    },
  ]

  const footerContent = (
    <>
      {withdrawal.status === WithdrawalStatus.PendingConfirmation && (
        <>
          <Button
            onClick={() => setOpenWithdrawalConfirmDialog(withdrawal)}
            variant="outline"
          >
            Confirm
          </Button>
          <Button
            variant="outline"
            onClick={() => setOpenWithdrawalCancelDialog(withdrawal)}
          >
            Cancel
          </Button>
        </>
      )}
      {withdrawal?.approvalProcess.status === ApprovalProcessStatus.InProgress &&
        withdrawal.approvalProcess.subjectCanSubmitDecision && (
          <>
            <Button variant="outline" onClick={() => setOpenApprovalDialog(true)}>
              Approve
            </Button>
            <Button variant="outline" onClick={() => setOpenDenialDialog(true)}>
              Deny
            </Button>
          </>
        )}
    </>
  )

  return (
    <>
      <DetailsCard
        title="Withdrawal"
        details={details}
        footerContent={footerContent}
        errorMessage={withdrawal.approvalProcess.deniedReason}
        className="max-w-7xl m-auto"
      />
      <VotersCard approvalProcess={withdrawal.approvalProcess} />
      {openWithdrawalConfirmDialog && (
        <WithdrawalConfirmDialog
          refetch={refetch}
          withdrawalData={openWithdrawalConfirmDialog}
          openWithdrawalConfirmDialog={Boolean(openWithdrawalConfirmDialog)}
          setOpenWithdrawalConfirmDialog={() => setOpenWithdrawalConfirmDialog(null)}
        />
      )}
      {openWithdrawalCancelDialog && (
        <WithdrawalCancelDialog
          refetch={refetch}
          withdrawalData={openWithdrawalCancelDialog}
          openWithdrawalCancelDialog={Boolean(openWithdrawalCancelDialog)}
          setOpenWithdrawalCancelDialog={() => setOpenWithdrawalCancelDialog(null)}
        />
      )}
      <ApprovalDialog
        approvalProcess={withdrawal?.approvalProcess as ApprovalProcess}
        openApprovalDialog={openApprovalDialog}
        setOpenApprovalDialog={() => setOpenApprovalDialog(false)}
        refetch={refetch}
      />
      <DenialDialog
        approvalProcess={withdrawal?.approvalProcess as ApprovalProcess}
        openDenialDialog={openDenialDialog}
        setOpenDenialDialog={() => setOpenDenialDialog(false)}
        refetch={refetch}
      />
    </>
  )
}

export default WithdrawalDetailsCard
