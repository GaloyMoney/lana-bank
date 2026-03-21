"use client"

import React from "react"
import { gql } from "@apollo/client"
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

import { useCommitteeUserRemoveMutation } from "@/lib/graphql/generated"

gql`
  mutation CommitteeUserRemove($input: CommitteeUserRemoveInput!) {
    committeeUserRemove(input: $input) {
      committee {
        ...CommitteeFields
      }
    }
  }
`

type RemoveUserCommitteeDialogProps = {
  committeeId: string
  userId: string
  userEmail: string
  openRemoveUserDialog: boolean
  setOpenRemoveUserDialog: (isOpen: boolean) => void
}

export const RemoveUserCommitteeDialog: React.FC<RemoveUserCommitteeDialogProps> = ({
  committeeId,
  userId,
  userEmail,
  openRemoveUserDialog,
  setOpenRemoveUserDialog,
}) => {
  const t = useTranslations("Committees.CommitteeDetails.RemoveUserCommitteeDialog")
  const [removeUser, { loading }] = useCommitteeUserRemoveMutation()

  const handleRemove = async () => {
    try {
      const { data } = await removeUser({
        variables: {
          input: {
            committeeId,
            userId,
          },
        },
      })
      if (data?.committeeUserRemove.committee) {
        toast.success(t("success"))
        setOpenRemoveUserDialog(false)
      }
    } catch (error) {
      console.error("Error removing user from committee:", error)
      toast.error(t("errors.failed"))
    }
  }

  return (
    <Dialog open={openRemoveUserDialog} onOpenChange={setOpenRemoveUserDialog}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("title")}</DialogTitle>
        </DialogHeader>
        <p>{t("description", { userEmail })}</p>
        <DialogFooter>
          <Button variant="ghost" onClick={() => setOpenRemoveUserDialog(false)}>
            {t("buttons.cancel")}
          </Button>
          <Button onClick={handleRemove} loading={loading}>
            {t("buttons.remove")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
