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

type DisbursalDetailsDialogProps = {
  setOpenDialog: (isOpen: boolean) => void
  openDialog: boolean
  disbursal: NonNullable<
    GetCreditFacilityDetailsQuery["creditFacility"]
  >["disbursals"][number]
}

export const DisbursalDetailsDialog: React.FC<DisbursalDetailsDialogProps> = ({
  setOpenDialog,
  openDialog,
  disbursal,
}) => {
  const handleCloseDialog = () => {
    setOpenDialog(false)
  }

  return (
    <Dialog open={openDialog} onOpenChange={handleCloseDialog}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Disbursal Details</DialogTitle>
          <DialogDescription>View the details of this disbursal.</DialogDescription>
        </DialogHeader>
        <DetailsGroup>
          <DetailItem
            className="px-0"
            label="ID"
            value={disbursal.id.split("disbursal:")[1]}
          />
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
      </DialogContent>
    </Dialog>
  )
}
