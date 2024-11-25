"use client"
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
} from "@/ui/dialog"
import { Button } from "@/ui/button"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/ui/select"
import {
  PoliciesDocument,
  GetPolicyDetailsDocument,
  useCommitteesQuery,
  usePolicyAssignCommitteeMutation,
} from "@/lib/graphql/generated"
import { Input } from "@/ui/input"
import { Label } from "@/ui/label"

gql`
  mutation PolicyAssignCommittee($input: PolicyAssignCommitteeInput!) {
    policyAssignCommittee(input: $input) {
      policy {
        id
        policyId
        approvalProcessType
      }
    }
  }
`

type CommitteeAssignmentDialogProps = {
  policyId: string
  setOpenAssignDialog: (isOpen: boolean) => void
  openAssignDialog: boolean
  refetch?: () => void
}

export const CommitteeAssignmentDialog: React.FC<CommitteeAssignmentDialogProps> = ({
  policyId,
  setOpenAssignDialog,
  openAssignDialog,
  refetch,
}) => {
  const [assignCommittee, { loading, reset, error: assignCommitteeError }] =
    usePolicyAssignCommitteeMutation({
      refetchQueries: [GetPolicyDetailsDocument],
    })
  const { data: committeeData, loading: committeesLoading } = useCommitteesQuery({
    variables: { first: 100 },
  })

  const [selectedCommitteeId, setSelectedCommitteeId] = useState<string>("")
  const [threshold, setThreshold] = useState<number | null>(null)
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    if (!selectedCommitteeId || threshold === null) {
      setError("Please select a committee and set threshold")
      return
    }

    try {
      const { data } = await assignCommittee({
        variables: {
          input: {
            policyId,
            committeeId: selectedCommitteeId,
            threshold,
          },
        },
        refetchQueries: [PoliciesDocument, GetPolicyDetailsDocument],
      })

      if (data?.policyAssignCommittee.policy) {
        toast.success("Committee assigned to policy successfully")
        if (refetch) refetch()
        setOpenAssignDialog(false)
      } else {
        throw new Error("Failed to assign committee to policy. Please try again.")
      }
    } catch (error) {
      console.error("Error assigning committee to policy:", error)
      if (error instanceof Error) {
        setError(error.message)
      } else if (assignCommitteeError?.message) {
        setError(assignCommitteeError.message)
      } else {
        setError("An unexpected error occurred. Please try again.")
      }
      toast.error("Failed to assign committee to policy")
    }
  }

  const resetForm = () => {
    setSelectedCommitteeId("")
    setThreshold(null)
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
          <DialogTitle>Assign Committee to Policy</DialogTitle>
          <DialogDescription>
            Select a committee and set threshold for this policy
          </DialogDescription>
        </DialogHeader>
        <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
          <div>
            <Label htmlFor="committee-select">Select Committee</Label>
            <Select value={selectedCommitteeId} onValueChange={setSelectedCommitteeId}>
              <SelectTrigger>
                <SelectValue placeholder="Select a committee" />
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

          <div>
            <Label htmlFor="threshold-input">Threshold</Label>
            <Input
              id="threshold-input"
              type="number"
              value={threshold || ""}
              onChange={(e) =>
                setThreshold(e.target.value ? Number(e.target.value) : null)
              }
              placeholder="Enter threshold value"
              min="0"
              max="100"
            />
          </div>

          {error && <p className="text-destructive text-sm">{error}</p>}

          <DialogFooter>
            <Button
              type="submit"
              disabled={
                loading || committeesLoading || !selectedCommitteeId || threshold === null
              }
            >
              Assign Committee
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

export default CommitteeAssignmentDialog
