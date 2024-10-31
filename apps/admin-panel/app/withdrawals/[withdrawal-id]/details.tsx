"use client"

import { useState } from "react"
import { gql } from "@apollo/client"
import { useRouter } from "next/navigation"
import { FaBan, FaCheckCircle, FaQuestion } from "react-icons/fa"

import { WithdrawalStatusBadge } from "../status-badge"
import { WithdrawalConfirmDialog } from "../confirm"
import { WithdrawalCancelDialog } from "../cancel"

import {
  ApprovalProcess,
  ApprovalProcessStatus,
  useGetWithdrawalDetailsQuery,
  WithdrawalStatus,
} from "@/lib/graphql/generated"
import { DetailItem } from "@/components/details"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/primitive/card"
import { Separator } from "@/components/primitive/separator"
import { Button } from "@/components/primitive/button"
import Balance from "@/components/balance/balance"
import { formatRole } from "@/lib/utils"
import ApprovalDialog from "@/app/approval-process/approve"
import DenialDialog from "@/app/approval-process/deny"

gql`
  query GetWithdrawalDetails($id: UUID!) {
    withdrawal(id: $id) {
      customerId
      withdrawalId
      amount
      status
      reference
      subjectCanConfirm
      subjectCanCancel
      customer {
        email
        customerId
        applicantId
      }
      approvalProcess {
        approvalProcessId
        approvalProcessType
        createdAt
        subjectCanSubmitDecision
        status
        rules {
          ... on CommitteeThreshold {
            threshold
            committee {
              name
              currentMembers {
                email
                roles
              }
            }
          }
          ... on SystemApproval {
            autoApprove
          }
        }
        voters {
          stillEligible
          didVote
          didApprove
          didDeny
          user {
            userId
            email
            roles
          }
        }
      }
    }
  }
`

type LoanDetailsProps = { withdrawalId: string }

const WithdrawalDetailsCard: React.FC<LoanDetailsProps> = ({ withdrawalId }) => {
  const router = useRouter()

  const {
    data: withdrawalDetails,
    loading,
    error,
    refetch: refetchWithdrawal,
  } = useGetWithdrawalDetailsQuery({
    variables: { id: withdrawalId },
  })

  const [openWithdrawalCancelDialog, setOpenWithdrawalCancelDialog] =
    useState<WithdrawalWithCustomer | null>(null)
  const [openWithdrawalConfirmDialog, setOpenWithdrawalConfirmDialog] =
    useState<WithdrawalWithCustomer | null>(null)
  const [openApprovalDialog, setOpenApprovalDialog] = useState(false)
  const [openDenialDialog, setOpenDenialDialog] = useState(false)

  return (
    <>
      <Card>
        {loading ? (
          <CardContent className="pt-6">Loading...</CardContent>
        ) : error ? (
          <CardContent className="pt-6 text-destructive">{error.message}</CardContent>
        ) : withdrawalDetails?.withdrawal ? (
          <>
            <CardHeader className="flex flex-row justify-between items-center">
              <div>
                <h2 className="font-semibold leading-none tracking-tight">Withdrawal</h2>
                <p className="text-textColor-secondary text-sm mt-2">
                  {withdrawalDetails.withdrawal.withdrawalId}
                </p>
              </div>
              <div className="flex flex-col gap-2">
                <WithdrawalStatusBadge status={withdrawalDetails.withdrawal.status} />
              </div>
            </CardHeader>
            <Separator className="mb-6" />
            <CardContent>
              <div className="grid grid-rows-min">
                <DetailItem
                  label="Customer Email"
                  value={withdrawalDetails.withdrawal.customer?.email}
                />
                <DetailItem
                  label="Withdrawal ID"
                  value={withdrawalDetails.withdrawal.withdrawalId}
                />
                <DetailItem
                  label="Withdrawal Amount"
                  value={
                    <Balance
                      amount={withdrawalDetails.withdrawal.amount}
                      currency="usd"
                    />
                  }
                />
                <DetailItem
                  label="Withdrawal Reference"
                  value={
                    withdrawalDetails.withdrawal.reference ===
                    withdrawalDetails.withdrawal.withdrawalId
                      ? "n/a"
                      : withdrawalDetails.withdrawal.reference
                  }
                />
              </div>
              <Separator className="my-6" />
              <div className="flex items-center justify-between">
                <Button
                  onClick={() =>
                    router.push(`/customers/${withdrawalDetails.withdrawal?.customerId}`)
                  }
                  className=""
                >
                  Show Customer
                </Button>
                <div>
                  {withdrawalDetails.withdrawal.status ===
                    WithdrawalStatus.PendingConfirmation && (
                    <Button
                      onClick={() =>
                        withdrawalDetails.withdrawal &&
                        setOpenWithdrawalConfirmDialog(withdrawalDetails.withdrawal)
                      }
                      className="ml-2"
                    >
                      Confirm
                    </Button>
                  )}
                  {withdrawalDetails.withdrawal.status ===
                    WithdrawalStatus.PendingConfirmation && (
                    <Button
                      onClick={() =>
                        withdrawalDetails.withdrawal &&
                        setOpenWithdrawalCancelDialog(withdrawalDetails.withdrawal)
                      }
                      className="ml-2"
                    >
                      Cancel
                    </Button>
                  )}
                  {withdrawalDetails?.withdrawal?.approvalProcess.status ===
                    ApprovalProcessStatus.InProgress &&
                    withdrawalDetails?.withdrawal.approvalProcess.subjectCanSubmitDecision && (
                      <>
                        <Button
                          onClick={() => setOpenApprovalDialog(true)}
                          className="ml-2"
                        >
                          Approve
                        </Button>
                        <Button
                          onClick={() => setOpenDenialDialog(true)}
                          className="ml-2"
                        >
                          Deny
                        </Button>
                      </>
                    )}
                </div>
              </div>
            </CardContent>
          </>
        ) : (
          withdrawalId &&
          !withdrawalDetails?.withdrawal?.withdrawalId && (
            <CardContent className="pt-6">No withdrawal found with this ID</CardContent>
          )
        )}
      </Card>
      {withdrawalDetails?.withdrawal?.approvalProcess.rules.__typename ===
        "CommitteeThreshold" && (
        <Card className="mt-4">
          <CardHeader>
            <CardTitle className="text-primary font-normal">
              Approval process decision from the{" "}
              {withdrawalDetails.withdrawal.approvalProcess.rules.committee.name}{" "}
              Committee
            </CardTitle>
          </CardHeader>
          <CardContent>
            {withdrawalDetails.withdrawal.approvalProcess.voters
              .filter((voter) => {
                if (
                  withdrawalDetails.withdrawal?.approvalProcess.status ===
                    ApprovalProcessStatus.InProgress ||
                  ([
                    ApprovalProcessStatus.Approved,
                    ApprovalProcessStatus.Denied,
                  ].includes(
                    withdrawalDetails.withdrawal?.approvalProcess
                      .status as ApprovalProcessStatus,
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
          refetch={refetchWithdrawal}
          withdrawalData={openWithdrawalConfirmDialog}
          openWithdrawalConfirmDialog={Boolean(openWithdrawalConfirmDialog)}
          setOpenWithdrawalConfirmDialog={() => setOpenWithdrawalConfirmDialog(null)}
        />
      )}
      {openWithdrawalCancelDialog && (
        <WithdrawalCancelDialog
          refetch={refetchWithdrawal}
          withdrawalData={openWithdrawalCancelDialog}
          openWithdrawalCancelDialog={Boolean(openWithdrawalCancelDialog)}
          setOpenWithdrawalCancelDialog={() => setOpenWithdrawalCancelDialog(null)}
        />
      )}
      <ApprovalDialog
        approvalProcess={
          withdrawalDetails?.withdrawal?.approvalProcess as ApprovalProcess
        }
        openApprovalDialog={openApprovalDialog}
        setOpenApprovalDialog={() => {
          setOpenApprovalDialog(false)
        }}
        refetch={refetchWithdrawal}
      />
      <DenialDialog
        approvalProcess={
          withdrawalDetails?.withdrawal?.approvalProcess as ApprovalProcess
        }
        openDenialDialog={openDenialDialog}
        setOpenDenialDialog={() => {
          setOpenDenialDialog(false)
        }}
        refetch={refetchWithdrawal}
      />
    </>
  )
}

export default WithdrawalDetailsCard
