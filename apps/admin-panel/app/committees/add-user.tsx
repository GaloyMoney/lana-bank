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

import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"

import {
  User,
  SortDirection,
  UsersSort,
  useCommitteeAddUserMutation,
  useUsersQuery,
} from "@/lib/graphql/generated"

import { camelToScreamingSnake } from "@/lib/utils"

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
  const [sortBy, setSortBy] = useState<UsersSort | null>(null)

  const { data: userData, loading: usersLoading, fetchMore } = useUsersQuery({
    variables: { first: DEFAULT_PAGESIZE, sort: sortBy },
    skip: !openAddUserDialog,
  })

  const [selectedUser, setSelectedUser] = useState<{
    userId: string
    email: string
  } | null>(null)
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    if (!selectedUser) {
      setError(t("errors.selectUser"))
      return
    }

    try {
      const { data } = await addUser({
        variables: {
          input: {
            committeeId,
            userId: selectedUser.userId,
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
    setSelectedUser(null)
    setError(null)
    reset()
  }

  const columns: Column<User>[] = [
    {
      key: "email",
      label: t("columns.email"),
      sortable: true,
    },
    {
      key: "role",
      label: t("columns.role"),
      render: (role) => role?.name || "",
    },
  ]

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
      <DialogContent className="sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>{t("title")}</DialogTitle>
          <DialogDescription>{t("description")}</DialogDescription>
        </DialogHeader>
        <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
          {selectedUser && (
            <p className="text-sm text-muted-foreground">
              {t("selectedUser")}: <strong>{selectedUser.email}</strong>
            </p>
          )}

          <PaginatedTable<User>
            columns={columns}
            data={userData?.users as PaginatedData<User>}
            loading={usersLoading}
            fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
            pageSize={DEFAULT_PAGESIZE}
            style="compact"
            onClick={(user) => setSelectedUser({ userId: user.userId, email: user.email })}
            onSort={(column, direction) => {
              setSortBy({
                by: camelToScreamingSnake(column as string) as UsersSort["by"],
                direction: direction as SortDirection,
              })
            }}
          />

          {error && <p className="text-destructive text-sm">{error}</p>}

          <DialogFooter>
            <Button
              type="submit"
              data-testid="committee-add-user-submit-button"
              disabled={loading || usersLoading || !selectedUser}
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
