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
import {
  Role,
  UsersDocument,
  useUserAssignRoleMutation,
  useUserCreateMutation,
} from "@/lib/graphql/generated"
import { Input } from "@/ui/input"
import { Button } from "@/ui/button"
import { Label } from "@/ui/label"
import { sendMagicLinkToEmail } from "@/lib/user/server-actions/send-magic-link"
import { formatRole } from "@/lib/utils"
import { Checkbox } from "@/ui/check-box"
import { useModalNavigation } from "@/hooks/use-modal-navigation"

gql`
  mutation UserCreate($input: UserCreateInput!) {
    userCreate(input: $input) {
      user {
        userId
        email
        roles
      }
    }
  }
`

type CreateUserDialogProps = {
  setOpenCreateUserDialog: (isOpen: boolean) => void
  openCreateUserDialog: boolean
  refetch?: () => void
}

export const CreateUserDialog: React.FC<CreateUserDialogProps> = ({
  setOpenCreateUserDialog,
  openCreateUserDialog,
  refetch,
}) => {
  const { navigate, isNavigating } = useModalNavigation({
    closeModal: () => setOpenCreateUserDialog(false),
  })

  const [createUser, { loading: creatingUser, reset: resetCreateUser }] =
    useUserCreateMutation({
      refetchQueries: [UsersDocument],
    })
  const [assignRole, { loading: assigningRole, reset: resetAssignRole }] =
    useUserAssignRoleMutation({
      refetchQueries: [UsersDocument],
    })

  const [email, setEmail] = useState("")
  const [selectedRoles, setSelectedRoles] = useState<Role[]>([])
  const [error, setError] = useState<string | null>(null)
  const [assignRoleError, setAssignRoleError] = useState<string | null>(null)
  const [isSendingMagicLink, setIsSendingMagicLink] = useState(false)

  const isLoading = creatingUser || assigningRole || isSendingMagicLink || isNavigating
  const isSubmitDisabled = isLoading || selectedRoles.length === 0

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    setAssignRoleError(null)

    if (selectedRoles.length === 0) {
      setError("Please select at least one role.")
      return
    }

    try {
      const result = await createUser({
        variables: { input: { email } },
      })

      if (result.data) {
        const userId = result.data.userCreate.user.userId
        await assignUserRoles(userId)
        await sendMagicLinkAndFinalize(userId)
      }
    } catch (error) {
      handleError(error, "Error creating user:")
    }
  }

  const assignUserRoles = async (userId: string) => {
    for (const role of selectedRoles) {
      try {
        await assignRole({
          variables: { input: { id: userId, role } },
        })
      } catch (error) {
        handleError(error, "Error assigning role:")
        return false
      }
    }
    return true
  }

  const sendMagicLinkAndFinalize = async (userId: string) => {
    setIsSendingMagicLink(true)
    try {
      await sendMagicLinkToEmail(email)
      toast.success("Magic link sent successfully")
      finalize(userId)
    } catch (error) {
      console.error("Error sending magic link:", error)
      toast.error("Failed to send magic link. Please try again later.")
    } finally {
      setIsSendingMagicLink(false)
    }
  }

  const finalize = (userId: string) => {
    if (refetch) refetch()
    navigate(`/users/${userId}`)
  }

  const handleError = (error: unknown, prefix: string) => {
    console.error(prefix, error)
    const errorMessage =
      error instanceof Error ? error.message : "An unknown error occurred"
    setError(`${prefix} ${errorMessage}`)
  }

  const resetStates = () => {
    setEmail("")
    setSelectedRoles([])
    setError(null)
    setAssignRoleError(null)
    setIsSendingMagicLink(false)
    resetCreateUser()
    resetAssignRole()
  }

  const handleRoleToggle = (role: Role) => {
    setSelectedRoles((prevRoles) =>
      prevRoles.includes(role)
        ? prevRoles.filter((r) => r !== role)
        : [...prevRoles, role],
    )
  }

  return (
    <Dialog
      open={openCreateUserDialog}
      onOpenChange={(isOpen) => {
        setOpenCreateUserDialog(isOpen)
        if (!isOpen) resetStates()
      }}
    >
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Add new User</DialogTitle>
          <DialogDescription>
            Add a new user to the admin-panel by providing their email address and
            selecting roles
          </DialogDescription>
        </DialogHeader>
        <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
          <div>
            <Label>Email</Label>
            <Input
              type="email"
              required
              placeholder="Please enter the email address"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              disabled={isLoading}
              data-testid="create-user-email-input"
            />
            <p className="text-textColor-secondary text-xs ml-1 mt-1.5">
              A magic link will be sent to the email address provided.
            </p>
          </div>

          <div>
            <Label>Roles</Label>
            <div className="ml-1 flex flex-col gap-1 align-middle">
              {Object.values(Role)
                .filter((role) => role !== Role.Superuser)
                .map((role) => (
                  <div className="flex items-center" key={role}>
                    <Checkbox
                      data-testid={`create-user-role-${role.toLowerCase()}-checkbox`}
                      id={role}
                      checked={selectedRoles.includes(role)}
                      onCheckedChange={() => handleRoleToggle(role)}
                      disabled={isLoading}
                    />
                    <label htmlFor={role} className="ml-2">
                      {formatRole(role)}
                    </label>
                  </div>
                ))}
            </div>
          </div>
          {error && <p className="text-destructive">{error}</p>}
          {assignRoleError && <p className="text-destructive">{assignRoleError}</p>}
          <DialogFooter>
            <Button
              type="submit"
              loading={isLoading}
              disabled={isSubmitDisabled}
              data-testid="create-user-submit-button"
            >
              {isLoading ? "Processing..." : "Submit"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
