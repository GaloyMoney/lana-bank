import React from "react"
import { gql, useApolloClient } from "@apollo/client"
import { toast } from "sonner"
import { useTranslations } from "next-intl"
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@lana/web/ui/dialog"
import { Button } from "@lana/web/ui/button"

import { formatDate } from "@lana/web/utils"

import {
  GetCreditFacilityLayoutDetailsDocument,
  GetDisbursalDetailsDocument,
  GetWithdrawalDetailsDocument,
  GetCreditFacilityProposalLayoutDetailsDocument,
  useApprovalProcessApproveMutation,
  ApprovalProcessFieldsFragment,
} from "@/lib/graphql/generated"
import { DetailItem, DetailsGroup } from "@/components/details"
import { formatProcessType } from "@/lib/utils"

gql`
  fragment ApprovalProcessFields on ApprovalProcess {
    id
    approvalProcessId
    deniedReason
    approvalProcessType
    createdAt
    userCanSubmitDecision
    status
    rules {
      ... on CommitteeThreshold {
        threshold
        committee {
          name
          currentMembers {
            id
            email
            role {
              ...RoleFields
            }
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
        role {
          ...RoleFields
        }
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
  approvalProcess: ApprovalProcessFieldsFragment
}

export const ApprovalDialog: React.FC<ApprovalDialogProps> = ({
  setOpenApprovalDialog,
  openApprovalDialog,
  approvalProcess,
}) => {
  const t = useTranslations("Actions.ApprovalProcess.Approve")
  const tCommon = useTranslations("Common")

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
  const client = useApolloClient()
  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    try {
      await approveProcess({
        variables: {
          input: {
            approvalProcessId: approvalProcess.approvalProcessId,
          },
        },
        onCompleted: async () => {
          await client.refetchQueries({
            include: [
              GetCreditFacilityLayoutDetailsDocument,
              GetWithdrawalDetailsDocument,
              GetDisbursalDetailsDocument,
              GetCreditFacilityProposalLayoutDetailsDocument,
            ],
          })
          toast.success(t("success.processApproved"))
        },
      })
      setOpenApprovalDialog(false)
    } catch (error) {
      if (error instanceof Error) {
        setError(error.message)
      } else {
        setError(t("errors.unknown"))
      }
    }
  }

  return (
    <Dialog open={openApprovalDialog} onOpenChange={setOpenApprovalDialog}>
      <DialogContent>
        <form onSubmit={handleSubmit}>
          <DialogHeader>
            <DialogTitle>{t("title")}</DialogTitle>
          </DialogHeader>

          <div className="py-4">
            <input
              type="text"
              className="sr-only"
              autoFocus
              onKeyDown={(e) => {
                if (e.key === "Escape") {
                  e.preventDefault()
                  setOpenApprovalDialog(false)
                }
              }}
            />

            <DetailsGroup layout="horizontal">
              <DetailItem
                label={t("fields.processType")}
                value={formatProcessType(approvalProcess?.approvalProcessType)}
              />
              <DetailItem
                label={t("fields.createdAt")}
                value={formatDate(approvalProcess?.createdAt)}
              />
            </DetailsGroup>
            {error && <p className="text-destructive text-sm">{error}</p>}
          </div>

          <DialogFooter className="flex gap-2 sm:gap-0">
            <Button
              type="button"
              variant="ghost"
              onClick={() => setOpenApprovalDialog(false)}
              data-testid="approval-process-dialog-cancel-button"
            >
              {tCommon("cancel")}
            </Button>
            <Button
              type="submit"
              loading={loading}
              data-testid="approval-process-dialog-approve-button"
            >
              {t("buttons.approve")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

export default ApprovalDialog
