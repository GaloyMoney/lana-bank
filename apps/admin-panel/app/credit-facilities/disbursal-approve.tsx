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

type CreditFacilityDisbursalApproveDialogProps = {
  setOpenDialog: (isOpen: boolean) => void
  openDialog: boolean
  creditFacilityId: string
  disbursalIdx: number
  disbursal: NonNullable<
    GetCreditFacilityDetailsQuery["creditFacility"]
  >["disbursals"][number]
  refetch: () => void
}

export const CreditFacilityDisbursalApproveDialog: React.FC<
  CreditFacilityDisbursalApproveDialogProps
> = ({ setOpenDialog, openDialog, disbursal, refetch }) => {
  const handleCloseDialog = () => {
    setOpenDialog(false)
  }

  const [openDenialDialog, setOpenDenialDialog] = React.useState(false)
  const [openApprovalDialog, setOpenApprovalDialog] = React.useState(false)

  return (
    <Dialog open={openDialog} onOpenChange={handleCloseDialog}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Credit Facility Disbursal Approval Process</DialogTitle>
          <DialogDescription>
            Review the disbursal details before approving
          </DialogDescription>
        </DialogHeader>
        <DetailsGroup>
          <DetailItem
            className="px-0"
            label="Amount"
            value={<Balance amount={disbursal.amount} currency="usd" />}
          />
          <DetailItem
            className="px-0"
            label="Created"
            value={formatDate(disbursal.createdAt)}
          />
        </DetailsGroup>
        <>
          {disbursal.approvalProcess.rules.__typename === "CommitteeThreshold" && (
            <Card className="mt-4">
              <CardHeader>
                <CardTitle className="text-primary font-normal">
                  Approval process decision from the{" "}
                  {disbursal.approvalProcess.rules.committee.name} Committee
                </CardTitle>
              </CardHeader>
              <CardContent>
                {disbursal.approvalProcess.voters
                  .filter((voter) => {
                    if (
                      disbursal?.approvalProcess.status ===
                        ApprovalProcessStatus.InProgress ||
                      ([
                        ApprovalProcessStatus.Approved,
                        ApprovalProcessStatus.Denied,
                      ].includes(
                        disbursal?.approvalProcess.status as ApprovalProcessStatus,
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
          {disbursal.approvalProcess.status === ApprovalProcessStatus.InProgress &&
            disbursal.approvalProcess.subjectCanSubmitDecision && (
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
        approvalProcess={disbursal.approvalProcess as ApprovalProcess}
        openApprovalDialog={openApprovalDialog}
        setOpenApprovalDialog={() => {
          setOpenApprovalDialog(false)
          handleCloseDialog()
        }}
        refetch={refetch}
      />
      <DenialDialog
        approvalProcess={disbursal.approvalProcess as ApprovalProcess}
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
