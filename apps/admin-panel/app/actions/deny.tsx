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

import { Textarea } from "@lana/web/ui/textarea"

import { formatDate } from "@lana/web/utils"

import {
  GetCreditFacilityLayoutDetailsDocument,
  GetDisbursalDetailsDocument,
  GetWithdrawalDetailsDocument,
  GetCreditFacilityProposalLayoutDetailsDocument,
  useApprovalProcessDenyMutation,
  ApprovalProcessFieldsFragment,
} from "@/lib/graphql/generated"
import { DetailItem, DetailsGroup } from "@/components/details"
import { formatProcessType } from "@/lib/utils"

gql`
  mutation ApprovalProcessDeny($input: ApprovalProcessDenyInput!, $reason: String!) {
    approvalProcessDeny(input: $input, reason: $reason) {
      approvalProcess {
        ...ApprovalProcessFields
      }
    }
  }
`

type DenialDialogProps = {
  setOpenDenialDialog: (isOpen: boolean) => void
  openDenialDialog: boolean
  approvalProcess: ApprovalProcessFieldsFragment
}

export const DenialDialog: React.FC<DenialDialogProps> = ({
  setOpenDenialDialog,
  openDenialDialog,
  approvalProcess,
}) => {
  const t = useTranslations("Actions.ApprovalProcess.Deny")
  const tCommon = useTranslations("Common")

  const client = useApolloClient()
  const [error, setError] = React.useState<string | null>(null)
  const [reason, setReason] = React.useState("")
  const [denyProcess, { loading }] = useApprovalProcessDenyMutation({
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

  const handleDeny = async () => {
    setError(null)
    if (!reason.trim()) {
      setError(t("errors.reasonRequired"))
      return
    }

    try {
      await denyProcess({
        variables: {
          input: {
            approvalProcessId: approvalProcess.approvalProcessId,
          },
          reason: reason.trim(),
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
          toast.success(t("success.processDenied"))
        },
      })
      setOpenDenialDialog(false)
    } catch (error) {
      if (error instanceof Error) {
        setError(error.message)
      } else {
        setError(t("errors.unknown"))
      }
    }
  }

  return (
    <Dialog open={openDenialDialog} onOpenChange={setOpenDenialDialog}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("title")}</DialogTitle>
        </DialogHeader>
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
        <div className="space-y-2">
          <label htmlFor="reason" className="text-sm font-medium">
            {t("fields.reasonLabel")}
          </label>
          <Textarea
            id="reason"
            data-testid="approval-process-dialog-deny-reason"
            value={reason}
            onChange={(e) => setReason(e.target.value)}
            placeholder={t("placeholders.reason")}
            className="min-h-[100px]"
          />
        </div>
        {error && <p className="text-destructive text-sm">{error}</p>}
        <DialogFooter className="flex gap-2 sm:gap-0">
          <Button variant="ghost" onClick={() => setOpenDenialDialog(false)}>
            {tCommon("cancel")}
          </Button>
          <Button
            onClick={handleDeny}
            loading={loading}
            data-testid="approval-process-dialog-deny-button"
          >
            {t("buttons.deny")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

export default DenialDialog
