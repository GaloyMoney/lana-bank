import React from "react"
import { FaBan, FaCheckCircle, FaQuestion } from "react-icons/fa"

import ApprovalDialog from "../approval-process/approve"
import DenialDialog from "../approval-process/deny"

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/primitive/dialog"
import { Button } from "@/components/primitive/button"
import {
  ApprovalProcess,
  ApprovalProcessStatus,
  GetCreditFacilityDetailsQuery,
} from "@/lib/graphql/generated"
import Balance from "@/components/balance/balance"
import { formatDate, formatRole } from "@/lib/utils"
import { DetailItem, DetailsGroup } from "@/components/details"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/primitive/card"

type CreditFacilityDisbursementApproveDialogProps = {
  setOpenDialog: (isOpen: boolean) => void
  openDialog: boolean
  creditFacilityId: string
  disbursementIdx: number
  disbursement: NonNullable<
    GetCreditFacilityDetailsQuery["creditFacility"]
  >["disbursements"][number]
  refetch: () => void
}

export const CreditFacilityDisbursementApproveDialog: React.FC<
  CreditFacilityDisbursementApproveDialogProps
> = ({ setOpenDialog, openDialog, disbursement, refetch }) => {
  const handleCloseDialog = () => {
    setOpenDialog(false)
  }

  const [openDenialDialog, setOpenDenialDialog] = React.useState(false)
  const [openApprovalDialog, setOpenApprovalDialog] = React.useState(false)

  return (
    <Dialog open={openDialog} onOpenChange={handleCloseDialog}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Credit Facility Disbursement Approval Process</DialogTitle>
          <DialogDescription>
            Review the disbursement details before approving
          </DialogDescription>
        </DialogHeader>
        <DetailsGroup>
          <DetailItem
            className="px-0"
            label="Amount"
            value={<Balance amount={disbursement.amount} currency="usd" />}
          />
          <DetailItem
            className="px-0"
            label="Created"
            value={formatDate(disbursement.createdAt)}
          />
        </DetailsGroup>
        <>
          {disbursement.approvalProcess.rules.__typename === "CommitteeThreshold" && (
            <Card className="mt-4">
              <CardHeader>
                <CardTitle className="text-primary font-normal">
                  Approval process decision from the{" "}
                  {disbursement.approvalProcess.rules.committee.name} Committee
                </CardTitle>
              </CardHeader>
              <CardContent>
                {disbursement.approvalProcess.voters
                  .filter((voter) => {
                    if (
                      disbursement?.approvalProcess.status ===
                        ApprovalProcessStatus.InProgress ||
                      ([
                        ApprovalProcessStatus.Approved,
                        ApprovalProcessStatus.Denied,
                      ].includes(
                        disbursement?.approvalProcess.status as ApprovalProcessStatus,
                      ) &&
                        voter.didVote)
                    ) {
                      return true
                    }
                    return false
                  })
                  .map((voter) => (
                    <div
                      key={voter.user.userId}
                      className="flex items-center space-x-3 p-2"
                    >
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
        </>
        <DialogFooter>
          {disbursement.approvalProcess.status === ApprovalProcessStatus.InProgress &&
            disbursement.approvalProcess.subjectCanSubmitDecision && (
              <>
                <Button onClick={() => setOpenApprovalDialog(true)} className="ml-2">
                  Approve
                </Button>
                <Button onClick={() => setOpenDenialDialog(true)} className="ml-2">
                  Deny
                </Button>
              </>
            )}
        </DialogFooter>
      </DialogContent>
      <ApprovalDialog
        approvalProcess={disbursement.approvalProcess as ApprovalProcess}
        openApprovalDialog={openApprovalDialog}
        setOpenApprovalDialog={() => {
          setOpenApprovalDialog(false)
          handleCloseDialog()
        }}
        refetch={refetch}
      />
      <DenialDialog
        approvalProcess={disbursement.approvalProcess as ApprovalProcess}
        openDenialDialog={openDenialDialog}
        setOpenDenialDialog={() => {
          setOpenDenialDialog(false)
          handleCloseDialog()
        }}
        refetch={refetch}
      />
    </Dialog>
  )
}
