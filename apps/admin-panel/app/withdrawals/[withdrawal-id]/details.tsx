"use client"

import React, { useState } from "react"
import { useTranslations } from "next-intl"

import { Button } from "@lana/web/ui/button"

import { ExternalLinkIcon } from "lucide-react"

import { WithdrawalStatusBadge } from "../status-badge"

import { WithdrawalConfirmDialog } from "./confirm"
import { WithdrawalCancelDialog } from "./cancel"
import { WithdrawalRevertDialog } from "./revert"

import { DetailsCard, DetailItemProps } from "@/components/details"
import Balance from "@/components/balance/balance"
import {
  ApprovalProcessStatus,
  GetWithdrawalDetailsQuery,
  WithdrawalStatus,
} from "@/lib/graphql/generated"
import ApprovalDialog from "@/app/actions/approve"
import DenialDialog from "@/app/actions/deny"
import { VotersCard } from "@/app/disbursals/[disbursal-id]/voters"

type WithdrawalDetailsProps = {
  withdrawal: NonNullable<GetWithdrawalDetailsQuery["withdrawal"]>
}

const WithdrawalDetailsCard: React.FC<WithdrawalDetailsProps> = ({ withdrawal }) => {
  const t = useTranslations("Withdrawals.WithdrawDetails.WithdrawalDetailsCard")
  const [openWithdrawalCancelDialog, setOpenWithdrawalCancelDialog] = useState<
    GetWithdrawalDetailsQuery["withdrawal"] | null
  >(null)
  const [openWithdrawalConfirmDialog, setOpenWithdrawalConfirmDialog] = useState<
    GetWithdrawalDetailsQuery["withdrawal"] | null
  >(null)
  const [openWithdrawalRevertDialog, setOpenWithdrawalRevertDialog] = useState<
    GetWithdrawalDetailsQuery["withdrawal"] | null
  >(null)
  const [openApprovalDialog, setOpenApprovalDialog] = useState(false)
  const [openDenialDialog, setOpenDenialDialog] = useState(false)

  const details: DetailItemProps[] = [
    {
      label: t("fields.customerEmail"),
      value: withdrawal.account.customer.email,
      href: `/customers/${withdrawal.account.customer.publicId}`,
    },
    {
      label: t("fields.withdrawalId"),
      value: (
        <a
          href={`https://cockpit.sumsub.com/checkus#/kyt/txns?search=${withdrawal.withdrawalId}`}
          target="_blank"
          rel="noopener noreferrer"
          className="flex items-center gap-1 text-blue-500 hover:underline"
          title={`Full ID: ${withdrawal.withdrawalId}`}
        >
          {`${withdrawal.withdrawalId.substring(0, 4)}...${withdrawal.withdrawalId.substring(
            withdrawal.withdrawalId.length - 4,
          )}`}
          <ExternalLinkIcon className="h-4 w-4" />
        </a>
      ),
    },
    {
      label: t("fields.withdrawalAmount"),
      value: <Balance amount={withdrawal.amount} currency="usd" />,
    },
    {
      label: t("fields.withdrawalReference"),
      value:
        withdrawal.reference === withdrawal.withdrawalId
          ? t("values.na")
          : withdrawal.reference,
    },
    {
      label: t("fields.status"),
      value: <WithdrawalStatusBadge status={withdrawal.status} />,
      valueTestId: "withdrawal-status-badge",
    },
  ]

  const footerContent = (
    <>
      {withdrawal.status === WithdrawalStatus.PendingConfirmation && (
        <>
          <Button
            onClick={() => setOpenWithdrawalConfirmDialog(withdrawal)}
            data-testid="withdraw-confirm-button"
            variant="outline"
          >
            {t("buttons.confirm")}
          </Button>
          <Button
            data-testid="withdraw-cancel-button"
            variant="outline"
            onClick={() => setOpenWithdrawalCancelDialog(withdrawal)}
          >
            {t("buttons.cancel")}
          </Button>
        </>
      )}
      {withdrawal.status === WithdrawalStatus.Confirmed && (
        <Button
          data-testid="withdraw-revert-button"
          variant="outline"
          onClick={() => setOpenWithdrawalRevertDialog(withdrawal)}
        >
          {t("buttons.revert")}
        </Button>
      )}
      {withdrawal?.approvalProcess.status === ApprovalProcessStatus.InProgress &&
        withdrawal.approvalProcess.userCanSubmitDecision && (
          <>
            <Button
              variant="outline"
              onClick={() => setOpenApprovalDialog(true)}
              data-testid="approval-process-approve-button"
            >
              {t("buttons.approve")}
            </Button>
            <Button
              variant="outline"
              onClick={() => setOpenDenialDialog(true)}
              data-testid="approval-process-deny-button"
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
        footerContent={footerContent}
        errorMessage={withdrawal.approvalProcess.deniedReason}
        className="max-w-7xl m-auto"
      />
      <VotersCard approvalProcess={withdrawal.approvalProcess} />
      {openWithdrawalConfirmDialog && (
        <WithdrawalConfirmDialog
          withdrawalData={openWithdrawalConfirmDialog}
          openWithdrawalConfirmDialog={Boolean(openWithdrawalConfirmDialog)}
          setOpenWithdrawalConfirmDialog={() => setOpenWithdrawalConfirmDialog(null)}
        />
      )}
      {openWithdrawalCancelDialog && (
        <WithdrawalCancelDialog
          withdrawalData={openWithdrawalCancelDialog}
          openWithdrawalCancelDialog={Boolean(openWithdrawalCancelDialog)}
          setOpenWithdrawalCancelDialog={() => setOpenWithdrawalCancelDialog(null)}
        />
      )}
      {openWithdrawalRevertDialog && (
        <WithdrawalRevertDialog
          withdrawalData={openWithdrawalRevertDialog}
          openWithdrawalRevertDialog={Boolean(openWithdrawalRevertDialog)}
          setOpenWithdrawalRevertDialog={() => setOpenWithdrawalRevertDialog(null)}
        />
      )}
      <ApprovalDialog
        approvalProcess={withdrawal?.approvalProcess}
        openApprovalDialog={openApprovalDialog}
        setOpenApprovalDialog={() => setOpenApprovalDialog(false)}
      />
      <DenialDialog
        approvalProcess={withdrawal?.approvalProcess}
        openDenialDialog={openDenialDialog}
        setOpenDenialDialog={() => setOpenDenialDialog(false)}
      />
    </>
  )
}

export default WithdrawalDetailsCard
