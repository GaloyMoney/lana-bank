"use client"

import React, { useEffect, useState } from "react"
import { useTranslations } from "next-intl"
import { toast } from "sonner"
import { gql } from "@apollo/client"

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
  useRolesQuery,
  useUserAssignRoleMutation,
  useUserRevokeRoleMutation,
} from "@/lib/graphql/generated"

gql`
  mutation UserAssignRole($input: UserAssignRoleInput!) {
    userAssignRole(input: $input) {
      user {
        ...UserFields
      }
    }
  }

  mutation UserRevokeRole($input: UserRevokeRoleInput!) {
    userRevokeRole(input: $input) {
      user {
        ...UserFields
      }
    }
  }
`

type UpdateUserRoleDialogProps = {
  open: boolean
  onOpenChange: (isOpen: boolean) => void
  userId: string
  currentRoleId?: string | null
}

export function UpdateUserRoleDialog({
  open,
  onOpenChange,
  userId,
  currentRoleId,
}: UpdateUserRoleDialogProps) {
  const t = useTranslations("Users.updateRole")
  const tCommon = useTranslations("Common")

  const [selectedRoleId, setSelectedRoleId] = useState<string>("placeholder")
  const [error, setError] = useState<string | null>(null)

  const { data, loading: rolesLoading } = useRolesQuery({
    variables: { first: 100 },
    skip: !open,
  })

  const roles = data?.roles.edges.map((edge) => edge.node) || []

  const [assignRole, { loading: assignLoading }] = useUserAssignRoleMutation()
  const [revokeRole, { loading: revokeLoading }] = useUserRevokeRoleMutation()
  const isLoading = rolesLoading || assignLoading || revokeLoading

  useEffect(() => {
    if (open) {
      setSelectedRoleId("placeholder")
      setError(null)
    }
  }, [open])

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    const newRoleId = selectedRoleId
    if (newRoleId === currentRoleId || !newRoleId) return

    try {
      if (currentRoleId && currentRoleId !== newRoleId) {
        await revokeRole({
          variables: {
            input: {
              id: userId,
              roleId: currentRoleId,
            },
          },
        })
      }

      if (newRoleId && newRoleId !== currentRoleId) {
        await assignRole({
          variables: {
            input: {
              id: userId,
              roleId: newRoleId,
            },
          },
        })
      }

      toast.success(t("success"))
      onOpenChange(false)
    } catch (error) {
      console.error("Failed to update role:", error)
      setError(t("error"))
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>{t("title")}</DialogTitle>
          <DialogDescription>{t("description")}</DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit}>
          <div>
            <Label htmlFor="role">{t("roleLabel")}</Label>
            <Select
              value={selectedRoleId}
              onValueChange={(value) => setSelectedRoleId(value)}
              disabled={isLoading}
            >
              <SelectTrigger className="w-full">
                <SelectValue placeholder={t("selectRole")} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="placeholder" disabled>
                  {t("selectRole")}
                </SelectItem>
                {roles.map((role) => (
                  <SelectItem key={role.roleId} value={role.roleId}>
                    {role.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          {error && <p className="text-destructive mt-2">{error}</p>}
          <DialogFooter className="mt-4">
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
              disabled={isLoading}
            >
              {tCommon("cancel")}
            </Button>
            <Button
              type="submit"
              loading={isLoading}
              disabled={isLoading || selectedRoleId === "placeholder"}
            >
              {isLoading ? tCommon("loading") : tCommon("update")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
