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
import { ApprovalProcess, useApprovalProcessDenyMutation } from "@/lib/graphql/generated"
import { DetailItem, DetailsGroup } from "@/components/details"
import { formatDate, formatProcessType } from "@/lib/utils"

gql`
  mutation ApprovalProcessDeny($input: ApprovalProcessDenyInput!) {
    approvalProcessDeny(input: $input) {
      approvalProcess {
        id
        approvalProcessId
        processType
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
  const [denyProcess, { loading }] = useApprovalProcessDenyMutation()

  const handleDeny = async () => {
    setError(null)
    try {
      await denyProcess({
        variables: {
          input: {
            processId: approvalProcess.approvalProcessId,
          },
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
            value={formatProcessType(approvalProcess?.processType)}
          />
          <DetailItem label="Created At" value={formatDate(approvalProcess?.createdAt)} />
        </DetailsGroup>
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
