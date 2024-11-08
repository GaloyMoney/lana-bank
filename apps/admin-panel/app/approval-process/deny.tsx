import React from "react"
import { gql } from "@apollo/client"
import { toast } from "sonner"

import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/primitive/dialog"
import { Button } from "@/components/primitive/button"
import {
  ApprovalProcess,
  CreditFacilitiesDocument,
  DisbursalsDocument,
  useApprovalProcessDenyMutation,
  WithdrawalsDocument,
} from "@/lib/graphql/generated"
import { DetailItem, DetailsGroup } from "@/components/details"
import { formatDate, formatProcessType } from "@/lib/utils"
import { Textarea } from "@/components/primitive/textarea"

gql`
  mutation ApprovalProcessDeny($input: ApprovalProcessDenyInput!, $reason: String!) {
    approvalProcessDeny(input: $input, reason: $reason) {
      approvalProcess {
        id
        approvalProcessId
        approvalProcessType
        createdAt
      }
    }
  }
`

type DenialDialogProps = {
  setOpenDenialDialog: (isOpen: boolean) => void
  openDenialDialog: boolean
  approvalProcess: ApprovalProcess
  refetch?: () => void
}

export const DenialDialog: React.FC<DenialDialogProps> = ({
  setOpenDenialDialog,
  openDenialDialog,
  refetch,
  approvalProcess,
}) => {
  const [error, setError] = React.useState<string | null>(null)
  const [reason, setReason] = React.useState("")
  const [denyProcess, { loading }] = useApprovalProcessDenyMutation({
    refetchQueries: [CreditFacilitiesDocument, WithdrawalsDocument, DisbursalsDocument],
  })

  const handleDeny = async () => {
    setError(null)
    if (!reason.trim()) {
      setError("Please provide a reason for denial")
      return
    }

    try {
      await denyProcess({
        variables: {
          input: {
            processId: approvalProcess.approvalProcessId,
          },
          reason: reason.trim(),
        },
        onCompleted: () => {
          if (refetch) refetch()
          toast.success("Process denied successfully")
        },
      })
      setOpenDenialDialog(false)
    } catch (error) {
      if (error instanceof Error) {
        setError(error.message)
      } else {
        setError("An unknown error occurred")
      }
    }
  }

  return (
    <Dialog open={openDenialDialog} onOpenChange={setOpenDenialDialog}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Deny Process</DialogTitle>
        </DialogHeader>
        <DetailsGroup>
          <DetailItem
            label="Process Type"
            value={formatProcessType(approvalProcess?.approvalProcessType)}
          />
          <DetailItem label="Created At" value={formatDate(approvalProcess?.createdAt)} />
        </DetailsGroup>
        <div className="space-y-2">
          <label htmlFor="reason" className="text-sm font-medium">
            Reason for Denial
          </label>
          <Textarea
            id="reason"
            value={reason}
            onChange={(e) => setReason(e.target.value)}
            placeholder="Please provide a reason for denying this process"
            className="min-h-[100px]"
          />
        </div>
        {error && <p className="text-destructive text-sm">{error}</p>}
        <DialogFooter className="flex gap-2 sm:gap-0">
          <Button variant="ghost" onClick={() => setOpenDenialDialog(false)}>
            Cancel
          </Button>
          <Button onClick={handleDeny} loading={loading}>
            Deny
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

export default DenialDialog
