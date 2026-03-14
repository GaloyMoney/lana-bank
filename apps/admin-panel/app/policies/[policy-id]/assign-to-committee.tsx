"use client"
import { useTranslations } from "next-intl"
import React, { useState } from "react"
import { gql } from "@apollo/client"
import { toast } from "sonner"

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@lana/web/ui/dialog"
import { Button } from "@lana/web/ui/button"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@lana/web/ui/select"

import { Label } from "@lana/web/ui/label"

import {
  useCommitteesQuery,
  usePolicyAssignCommitteeMutation,
} from "@/lib/graphql/generated"

gql`
  mutation PolicyAssignCommittee($input: PolicyAssignCommitteeInput!) {
    policyAssignCommittee(input: $input) {
      policy {
        id
        policyId
        approvalProcessType
        rules {
          ... on CommitteeApproval {
            committee {
              ...CommitteeFields
            }
          }
          ... on AutoApproval {
            __typename
          }
        }
      }
    }
  }
`

type CommitteeAssignmentDialogProps = {
  policyId: string
  setOpenAssignDialog: (isOpen: boolean) => void
  openAssignDialog: boolean
}

export const CommitteeAssignmentDialog: React.FC<CommitteeAssignmentDialogProps> = ({
  policyId,
  setOpenAssignDialog,
  openAssignDialog,
}) => {
  const t = useTranslations("Policies.PolicyDetails.CommitteeAssignmentDialog")

  const [assignCommittee, { loading, reset, error: assignCommitteeError }] =
    usePolicyAssignCommitteeMutation()
  const { data: committeeData, loading: committeesLoading } = useCommitteesQuery({
    variables: { first: 100 },
  })

  const [selectedCommitteeId, setSelectedCommitteeId] = useState<string>("")
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    if (!selectedCommitteeId) {
      setError(t("errors.selectCommittee"))
      return
    }

    try {
      const { data } = await assignCommittee({
        variables: {
          input: {
            policyId,
            committeeId: selectedCommitteeId,
          },
        },
      })

      if (data?.policyAssignCommittee.policy) {
        toast.success(t("success.assigned"))
        setOpenAssignDialog(false)
      } else {
        throw new Error(t("errors.assignmentFailed"))
      }
    } catch (error) {
      console.error("Error assigning committee to policy:", error)
      if (error instanceof Error) {
        setError(error.message)
      } else if (assignCommitteeError?.message) {
        setError(assignCommitteeError.message)
      } else {
        setError(t("errors.general"))
      }
      toast.error(t("errors.assignmentFailed"))
    }
  }

  const resetForm = () => {
    setSelectedCommitteeId("")
    setError(null)
    reset()
  }

  return (
    <Dialog
      open={openAssignDialog}
      onOpenChange={(isOpen) => {
        setOpenAssignDialog(isOpen)
        if (!isOpen) {
          resetForm()
        }
      }}
    >
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{t("title")}</DialogTitle>
          <DialogDescription>{t("description")}</DialogDescription>
        </DialogHeader>
        <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
          <div>
            <Label htmlFor="committee-select">{t("fields.committee")}</Label>
            <Select value={selectedCommitteeId} onValueChange={setSelectedCommitteeId}>
              <SelectTrigger data-testid="policy-select-committee-selector">
                <SelectValue placeholder={t("placeholders.committee")} />
              </SelectTrigger>
              <SelectContent>
                {committeeData?.committees.edges.map((edge) => (
                  <SelectItem key={edge.node.id} value={edge.node.committeeId}>
                    {edge.node.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {error && <p className="text-destructive text-sm">{error}</p>}

          <DialogFooter>
            <Button
              type="submit"
              data-testid="policy-assign-committee-submit-button"
              disabled={loading || committeesLoading || !selectedCommitteeId}
            >
              {t("buttons.assign")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

export default CommitteeAssignmentDialog
