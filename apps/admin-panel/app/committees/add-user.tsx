"use client"

import React, { useState } from "react"
import { gql } from "@apollo/client"
import { toast } from "sonner"
import { useTranslations } from "next-intl"

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

import { useCommitteeAddUserMutation, useUsersQuery } from "@/lib/graphql/generated"

gql`
  mutation CommitteeAddUser($input: CommitteeAddUserInput!) {
    committeeAddUser(input: $input) {
      committee {
        ...CommitteeFields
      }
    }
  }
`

type AddUserCommitteeDialogProps = {
  committeeId: string
  setOpenAddUserDialog: (isOpen: boolean) => void
  openAddUserDialog: boolean
}

export const AddUserCommitteeDialog: React.FC<AddUserCommitteeDialogProps> = ({
  committeeId,
  setOpenAddUserDialog,
  openAddUserDialog,
}) => {
  const t = useTranslations("Committees.CommitteeDetails.AddUserCommitteeDialog")
  const [addUser, { loading, reset, error: addUserError }] = useCommitteeAddUserMutation()
  const { data: userData, loading: usersLoading } = useUsersQuery({ variables: { first: 100 } })

  const [selectedUserId, setSelectedUserId] = useState<string>("")
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    if (!selectedUserId) {
      setError(t("errors.selectUser"))
      return
    }

    try {
      const { data } = await addUser({
        variables: {
          input: {
            committeeId,
            userId: selectedUserId,
          },
        },
      })

      if (data?.committeeAddUser.committee) {
        toast.success(t("success"))
        setOpenAddUserDialog(false)
      } else {
        throw new Error(t("errors.failed"))
      }
    } catch (error) {
      console.error("Error adding user to committee:", error)
      setError(addUserError?.message || t("errors.general"))
      toast.error(t("errors.failed"))
    }
  }

  const resetForm = () => {
    setSelectedUserId("")
    setError(null)
    reset()
  }

  return (
    <Dialog
      open={openAddUserDialog}
      onOpenChange={(isOpen) => {
        setOpenAddUserDialog(isOpen)
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
          <Select value={selectedUserId} onValueChange={setSelectedUserId}>
            <SelectTrigger data-testid="committee-add-user-select">
              <SelectValue placeholder={t("placeholders.selectUser")} />
            </SelectTrigger>
            <SelectContent>
              {userData?.users?.edges?.map((edge) => {
                const user = edge.node;
                return (
                  <SelectItem key={user.userId} value={user.userId}>
                    {user.email} {user.role?.name}
                  </SelectItem>
                );
              })}
            </SelectContent>
          </Select>

          {error && <p className="text-destructive text-sm">{error}</p>}

          <DialogFooter>
            <Button
              type="submit"
              data-testid="committee-add-user-submit-button"
              disabled={loading || usersLoading || !selectedUserId}
            >
              {t("buttons.addUser")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

export default AddUserCommitteeDialog
