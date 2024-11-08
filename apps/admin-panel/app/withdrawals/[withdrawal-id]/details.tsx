"use client"
import { useState } from "react"
import { FaBan, FaCheckCircle, FaQuestion } from "react-icons/fa"
import Link from "next/link"

import { WithdrawalStatusBadge } from "../status-badge"
import { WithdrawalConfirmDialog } from "../confirm"
import { WithdrawalCancelDialog } from "../cancel"

import {
  ApprovalProcess,
  ApprovalProcessStatus,
  GetWithdrawalDetailsQuery,
  WithdrawalStatus,
} from "@/lib/graphql/generated"
import { DetailItem } from "@/components/details"
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/primitive/card"
import { Button } from "@/components/primitive/button"
import Balance from "@/components/balance/balance"
import { formatRole } from "@/lib/utils"
import ApprovalDialog from "@/app/approval-process/approve"
import DenialDialog from "@/app/approval-process/deny"

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

  return (
    <>
      <Card className="max-w-7xl m-auto">
        <CardHeader className="flex flex-row justify-between items-center">
          <div>
            <h2 className="font-semibold leading-none tracking-tight">Withdrawal</h2>
            <p className="text-textColor-secondary text-sm mt-2">
              {withdrawal.withdrawalId}
            </p>
          </div>
          <div className="flex flex-col gap-2">
            <WithdrawalStatusBadge status={withdrawal.status} />
          </div>
        </CardHeader>
        <CardContent>
          <div className="grid grid-rows-min">
            <Link href={`/customers/${withdrawal.customer.customerId}`}>
              <DetailItem
                hover={true}
                label="Customer Email"
                value={withdrawal.customer?.email}
              />
            </Link>

            <DetailItem
              label="Withdrawal Amount"
              value={<Balance amount={withdrawal.amount} currency="usd" />}
            />
            <DetailItem
              label="Withdrawal Reference"
              value={
                withdrawal.reference === withdrawal.withdrawalId
                  ? "n/a"
                  : withdrawal.reference
              }
            />
          </div>
          <div className="flex items-center justify-between mt-4">
            <div>
              {withdrawal.status === WithdrawalStatus.PendingConfirmation && (
                <Button
                  onClick={() => withdrawal && setOpenWithdrawalConfirmDialog(withdrawal)}
                  className="ml-2"
                >
                  Confirm
                </Button>
              )}
              {withdrawal.status === WithdrawalStatus.PendingConfirmation && (
                <Button
                  variant="outline"
                  onClick={() => withdrawal && setOpenWithdrawalCancelDialog(withdrawal)}
                  className="ml-2"
                >
                  Cancel
                </Button>
              )}
              {withdrawal?.approvalProcess.status === ApprovalProcessStatus.InProgress &&
                withdrawal.approvalProcess.subjectCanSubmitDecision && (
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
          </div>
        </CardContent>
        {withdrawal.approvalProcess.deniedReason && (
          <CardFooter className="text-destructive">
            Denied Reason: {withdrawal.approvalProcess.deniedReason}
          </CardFooter>
        )}
      </Card>

      {withdrawal?.approvalProcess.rules.__typename === "CommitteeThreshold" && (
        <Card className="mt-4">
          <CardHeader>
            <CardTitle className="text-primary font-normal">
              Approval process decision from the{" "}
              {withdrawal.approvalProcess.rules.committee.name} Committee
            </CardTitle>
          </CardHeader>
          <CardContent>
            {withdrawal.approvalProcess.voters
              .filter((voter) => {
                if (
                  withdrawal?.approvalProcess.status ===
                    ApprovalProcessStatus.InProgress ||
                  ([
                    ApprovalProcessStatus.Approved,
                    ApprovalProcessStatus.Denied,
                  ].includes(
                    withdrawal?.approvalProcess.status as ApprovalProcessStatus,
                  ) &&
                    voter.didVote)
                ) {
                  return true
                }
                return false
              })
              .map((voter) => (
                <div key={voter.user.userId} className="flex items-center space-x-3 p-2">
                  {voter.didApprove ? (
                    <FaCheckCircle className="h-6 w-6 text-green-500" />
                  ) : voter.didDeny ? (
                    <FaBan className="h-6 w-6 text-red-500" />
                  ) : !voter.didVote ? (
                    <FaQuestion className="h-6 w-6 text-textColor-secondary" />
                  ) : (
                    <>{/* Impossible */}</>
                  )}
                  <div>
                    <p className="text-sm font-medium">{voter.user.email}</p>
                    <p className="text-sm text-textColor-secondary">
                      {voter.user.roles.map(formatRole).join(", ")}
                    </p>
                    {
                      <p className="text-xs text-textColor-secondary">
                        {voter.didApprove && "Approved"}
                        {voter.didDeny && "Denied"}
                        {!voter.didVote && "Has not voted yet"}
                      </p>
                    }
                  </div>
                </div>
              ))}
          </CardContent>
        </Card>
      )}

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
        setOpenApprovalDialog={() => {
          setOpenApprovalDialog(false)
        }}
        refetch={refetch}
      />
      <DenialDialog
        approvalProcess={withdrawal?.approvalProcess as ApprovalProcess}
        openDenialDialog={openDenialDialog}
        setOpenDenialDialog={() => {
          setOpenDenialDialog(false)
        }}
        refetch={refetch}
      />
    </>
  )
}

export default WithdrawalDetailsCard
