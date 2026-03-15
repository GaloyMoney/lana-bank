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
import { Input } from "@lana/web/ui/input"
import { Label } from "@lana/web/ui/label"

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
import { useProcessTypeLabel } from "@/app/actions/hooks"
import { authenticateWithPassword } from "@/app/auth/step-up"

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
      ... on CommitteeApproval {
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
      ... on AutoApproval {
        __typename
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
  const processTypeLabel = useProcessTypeLabel()

  const [error, setError] = React.useState<string | null>(null)
  const [password, setPassword] = React.useState("")
  const [authenticating, setAuthenticating] = React.useState(false)
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

    if (!password.trim()) {
      setError(t("errors.passwordRequired"))
      return
    }

    setAuthenticating(true)
    let freshToken: string
    try {
      freshToken = await authenticateWithPassword(password)
    } catch {
      setError(t("errors.invalidPassword"))
      setAuthenticating(false)
      return
    }
    setAuthenticating(false)

    try {
      await approveProcess({
        variables: {
          input: {
            approvalProcessId: approvalProcess.approvalProcessId,
          },
        },
        context: {
          headers: {
            Authorization: `Bearer ${freshToken}`,
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

          <div className="py-4 space-y-4">
            <DetailsGroup layout="horizontal">
              <DetailItem
                label={t("fields.processType")}
                value={processTypeLabel(approvalProcess?.approvalProcessType)}
              />
              <DetailItem
                label={t("fields.createdAt")}
                value={formatDate(approvalProcess?.createdAt)}
              />
            </DetailsGroup>
            <div className="space-y-2">
              <Label htmlFor="step-up-password">{t("fields.passwordLabel")}</Label>
              <Input
                id="step-up-password"
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder={t("placeholders.password")}
                data-testid="approval-process-dialog-password"
                autoFocus
              />
            </div>
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
              loading={loading || authenticating}
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
