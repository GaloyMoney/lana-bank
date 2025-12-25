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

import { Input } from "@lana/web/ui/input"
import { Button } from "@lana/web/ui/button"
import { Label } from "@lana/web/ui/label"

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@lana/web/ui/select"

import { PermissionsDisplay } from "./permissions-display"

import { useRolesQuery, useUserCreateMutation } from "@/lib/graphql/generated"
import { useModalNavigation } from "@/hooks/use-modal-navigation"

gql`
  mutation UserCreate($input: UserCreateInput!) {
    userCreate(input: $input) {
      user {
        ...UserFields
      }
    }
  }
`

type CreateUserDialogProps = {
  setOpenCreateUserDialog: (isOpen: boolean) => void
  openCreateUserDialog: boolean
}

export const CreateUserDialog: React.FC<CreateUserDialogProps> = ({
  setOpenCreateUserDialog,
  openCreateUserDialog,
}) => {
  const t = useTranslations("Users.createDialog")
  const tCommon = useTranslations("Common")

  const closeDialog = () => {
    setOpenCreateUserDialog(false)
    resetStates()
  }

  const { navigate, isNavigating } = useModalNavigation({
    closeModal: closeDialog,
  })

  const { data: rolesData, loading: rolesLoading } = useRolesQuery({
    variables: { first: 100, after: null },
    skip: !openCreateUserDialog,
  })

  const roles = rolesData?.roles.edges.map((edge) => edge.node) || []

  const [createUser, { loading: creatingUser }] = useUserCreateMutation({
    update: (cache) => {
      cache.modify({
        fields: {
          users: (_, { DELETE }) => DELETE,
        },
      })
      cache.gc()
    },
  })

  const [email, setEmail] = useState("")
  const [selectedRoleId, setSelectedRoleId] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)

  const isLoading = creatingUser || isNavigating || rolesLoading
  const isSubmitDisabled = isLoading || !selectedRoleId

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    if (!selectedRoleId) {
      setError(t("errors.selectRole"))
      return
    }

    try {
      const result = await createUser({
        variables: {
          input: {
            email,
            roleId: selectedRoleId,
          },
        },
      })

      if (result.data) {
        const userId = result.data.userCreate.user.userId
        finalize(userId)
      }
    } catch (error) {
      handleError(error, t("errors.createPrefix"))
    }
  }

  const finalize = (userId: string) => {
    toast.success(t("success.userCreated"))
    navigate(`/users/${userId}`)
  }

  const handleError = (error: unknown, prefix: string) => {
    console.error(prefix, error)
    const errorMessage = error instanceof Error ? error.message : t("errors.unknown")
    setError(`${prefix} ${errorMessage}`)
  }

  const resetStates = () => {
    setEmail("")
    setSelectedRoleId(null)
    setError(null)
  }

  const selectedRole = selectedRoleId
    ? roles.find((role) => role.roleId === selectedRoleId)
    : null

  const permissionSets = selectedRole?.permissionSets || []

  return (
    <Dialog
      open={openCreateUserDialog}
      onOpenChange={(isOpen) => {
        setOpenCreateUserDialog(isOpen)
        if (!isOpen) resetStates()
      }}
    >
      <DialogContent className="max-w-xl">
        <DialogHeader>
          <DialogTitle>{t("title")}</DialogTitle>
          <DialogDescription>{t("description")}</DialogDescription>
        </DialogHeader>
        <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
          <div>
            <Label htmlFor="email">{t("fields.email")}</Label>
            <Input
              id="email"
              type="email"
              required
              placeholder={t("placeholders.email")}
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              disabled={isLoading}
              data-testid="create-user-email-input"
            />
          </div>

          <div>
            <Label htmlFor="role">{t("fields.role")}</Label>
            <Select
              value={selectedRoleId || ""}
              onValueChange={(value) => setSelectedRoleId(value || null)}
              disabled={isLoading}
            >
              <SelectTrigger id="role">
                <SelectValue placeholder={t("placeholders.selectRole")} />
              </SelectTrigger>
              <SelectContent>
                {rolesLoading ? (
                  <SelectItem value="loading" disabled>
                    {tCommon("loading")}
                  </SelectItem>
                ) : roles.length === 0 ? (
                  <SelectItem value="none" disabled>
                    {t("noRolesAvailable")}
                  </SelectItem>
                ) : (
                  roles.map((role) => (
                    <SelectItem
                      key={role.roleId}
                      value={role.roleId}
                      data-testid={`create-user-role-${role.name.toLowerCase()}-option`}
                    >
                      {role.name}
                    </SelectItem>
                  ))
                )}
              </SelectContent>
            </Select>
          </div>

          <PermissionsDisplay
            permissionSets={permissionSets}
            hasSelectedRole={!!selectedRoleId}
          />

          {error && <p className="text-destructive">{error}</p>}

          <DialogFooter>
            <Button
              type="submit"
              loading={isLoading}
              disabled={isSubmitDisabled}
              data-testid="create-user-submit-button"
            >
              {isLoading ? t("buttons.processing") : t("buttons.submit")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
