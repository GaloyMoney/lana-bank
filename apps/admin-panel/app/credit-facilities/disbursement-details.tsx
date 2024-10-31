import React from "react"
import { FaCheckCircle, FaBan, FaQuestion } from "react-icons/fa"

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/primitive/dialog"
import Balance from "@/components/balance/balance"
import { formatDate, formatRole } from "@/lib/utils"
import { DetailItem, DetailsGroup } from "@/components/details"
import {
  ApprovalProcessStatus,
  GetCreditFacilityDetailsQuery,
} from "@/lib/graphql/generated"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/primitive/card"

type DisbursementDetailsDialogProps = {
  setOpenDialog: (isOpen: boolean) => void
  openDialog: boolean
  disbursement: NonNullable<
    GetCreditFacilityDetailsQuery["creditFacility"]
  >["disbursements"][number]
}

export const DisbursementDetailsDialog: React.FC<DisbursementDetailsDialogProps> = ({
  setOpenDialog,
  openDialog,
  disbursement,
}) => {
  const handleCloseDialog = () => {
    setOpenDialog(false)
  }

  return (
    <Dialog open={openDialog} onOpenChange={handleCloseDialog}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Disbursement Details</DialogTitle>
          <DialogDescription>View the details of this disbursement.</DialogDescription>
        </DialogHeader>
        <DetailsGroup>
          <DetailItem
            className="px-0"
            label="ID"
            value={disbursement.id.split("disbursement:")[1]}
          />
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
      </DialogContent>
    </Dialog>
  )
}
