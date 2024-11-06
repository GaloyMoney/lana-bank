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
  useApprovalProcessApproveMutation,
} from "@/lib/graphql/generated"
import { DetailItem, DetailsGroup } from "@/components/details"
import { formatDate, formatProcessType } from "@/lib/utils"

gql`
  mutation ApprovalProcessApprove($input: ApprovalProcessApproveInput!) {
    approvalProcessApprove(input: $input) {
      approvalProcess {
        id
        approvalProcessId
        approvalProcessType
        createdAt
      }
    }
  }
`

type ApprovalDialogProps = {
  setOpenApprovalDialog: (isOpen: boolean) => void
  openApprovalDialog: boolean
  approvalProcess: ApprovalProcess
  refetch?: () => void
}

export const ApprovalDialog: React.FC<ApprovalDialogProps> = ({
  setOpenApprovalDialog,
  openApprovalDialog,
  approvalProcess,
  refetch,
}) => {
  const [error, setError] = React.useState<string | null>(null)
  const [approveProcess, { loading }] = useApprovalProcessApproveMutation()

  const handleApprove = async () => {
    setError(null)
    try {
      await approveProcess({
        variables: {
          input: {
            processId: approvalProcess.approvalProcessId,
          },
        },
        onCompleted: () => {
          if (refetch) refetch()
          toast.success("Process approved successfully")
        },
      })
      setOpenApprovalDialog(false)
    } catch (error) {
      if (error instanceof Error) {
        setError(error.message)
      } else {
        setError("An unknown error occurred")
      }
    }
  }

  return (
    <Dialog open={openApprovalDialog} onOpenChange={setOpenApprovalDialog}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Approve Process</DialogTitle>
        </DialogHeader>
        <DetailsGroup>
          <DetailItem
            label="Process Type"
            value={formatProcessType(approvalProcess?.approvalProcessType)}
          />
          <DetailItem label="Created At" value={formatDate(approvalProcess?.createdAt)} />
        </DetailsGroup>
        {error && <p className="text-destructive text-sm">{error}</p>}
        <DialogFooter className="flex gap-2 sm:gap-0">
          <Button variant="ghost" onClick={() => setOpenApprovalDialog(false)}>
            Cancel
          </Button>
          <Button onClick={handleApprove} loading={loading}>
            Approve
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

export default ApprovalDialog
