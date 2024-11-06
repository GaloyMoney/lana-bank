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
} from "@/components/primitive/dialog"
import { Button } from "@/components/primitive/button"
import {
  CommitteesDocument,
  GetCommitteeDetailsDocument,
  useCommitteeAddUserMutation,
  useUsersQuery,
} from "@/lib/graphql/generated"
import { Select } from "@/components/primitive/select"
import { formatRole } from "@/lib/utils"

gql`
  mutation CommitteeAddUser($input: CommitteeAddUserInput!) {
    committeeAddUser(input: $input) {
      committee {
        id
        committeeId
        currentMembers {
          userId
          email
          roles
        }
      }
    }
  }
`

type AddUserCommitteeDialogProps = {
  committeeId: string
  setOpenAddUserDialog: (isOpen: boolean) => void
  openAddUserDialog: boolean
  refetch?: () => void
}

export const AddUserCommitteeDialog: React.FC<AddUserCommitteeDialogProps> = ({
  committeeId,
  setOpenAddUserDialog,
  openAddUserDialog,
  refetch,
}) => {
  const [addUser, { loading, reset, error: addUserError }] = useCommitteeAddUserMutation()
  const { data: userData, loading: usersLoading } = useUsersQuery()

  const [selectedUserId, setSelectedUserId] = useState<string>("")
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    if (!selectedUserId) {
      setError("Please select a user")
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
        refetchQueries: [CommitteesDocument, GetCommitteeDetailsDocument],
      })

      if (data?.committeeAddUser.committee) {
        toast.success("User added to committee successfully")
        if (refetch) refetch()
        setOpenAddUserDialog(false)
      } else {
        throw new Error("Failed to add user to committee. Please try again.")
      }
    } catch (error) {
      console.error("Error adding user to committee:", error)
      if (error instanceof Error) {
        setError(error.message)
      } else if (addUserError?.message) {
        setError(addUserError.message)
      } else {
        setError("An unexpected error occurred. Please try again.")
      }
      toast.error("Failed to add user to committee")
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
          <DialogTitle>Add User to Committee</DialogTitle>
          <DialogDescription>Select a user to add to this committee</DialogDescription>
        </DialogHeader>
        <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
          <Select
            value={selectedUserId}
            onChange={(e) => setSelectedUserId(e.target.value)}
          >
            <option value="">Select a user</option>
            {userData?.users.map((user) => (
              <option key={user.userId} value={user.userId}>
                {user.email} ({formatRole(user.roles.map(formatRole).join(", "))})
              </option>
            ))}
          </Select>

          {error && <p className="text-destructive text-sm">{error}</p>}

          <DialogFooter>
            <Button type="submit" disabled={loading || usersLoading || !selectedUserId}>
              Add User
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

export default AddUserCommitteeDialog
