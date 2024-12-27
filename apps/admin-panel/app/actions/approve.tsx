import React from "react"
import { gql } from "@apollo/client"
import { toast } from "sonner"

import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/ui/dialog"
import { Button } from "@/ui/button"
import {
  ApprovalProcess,
  useApprovalProcessApproveMutation,
} from "@/lib/graphql/generated"
import { DetailItem, DetailsGroup } from "@/components/details"
import { formatDate, formatProcessType } from "@/lib/utils"

gql`
  fragment ApprovalProcessFields on ApprovalProcess {
    id
    approvalProcessId
    deniedReason
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
            id
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
        id
        userId
        email
        roles
      }
    }
  }

  mutation ApprovalProcessApprove($input: ApprovalProcessApproveInput!) {
    approvalProcessApprove(input: $input) {
      approvalProcess {
        ...ApprovalProcessFields
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
  const [approveProcess, { loading }] = useApprovalProcessApproveMutation({
    update: (cache) => {
      cache.modify({
        fields: {
          creditFacilities: (_, { DELETE }) => DELETE,
          withdrawals: (_, { DELETE }) => DELETE,
          disbursals: (_, { DELETE }) => DELETE,
        },
      })
      cache.gc()
    },
  })

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
        <DetailsGroup layout="horizontal">
          <DetailItem
            label="Process Type"
            value={formatProcessType(approvalProcess?.approvalProcessType)}
          />
          <DetailItem label="Created At" value={formatDate(approvalProcess?.createdAt)} />
        </DetailsGroup>
        {error && <p className="text-destructive text-sm">{error}</p>}
        <DialogFooter className="flex gap-2 sm:gap-0">
          <Button
            variant="ghost"
            onClick={() => setOpenApprovalDialog(false)}
            data-testid="approval-process-dialog-cancel-button"
          >
            Cancel
          </Button>
          <Button
            onClick={handleApprove}
            loading={loading}
            data-testid="approval-process-dialog-approve-button"
          >
            Approve
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

export default ApprovalDialog
