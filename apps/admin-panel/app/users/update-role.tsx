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

import { PermissionsDisplay } from "./permissions-display"

import { useRolesQuery, useUserUpdateRoleMutation } from "@/lib/graphql/generated"

gql`
  mutation UserUpdateRole($input: UserUpdateRoleInput!) {
    userUpdateRole(input: $input) {
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

  const [selectedRoleId, setSelectedRoleId] = useState<string | undefined>(undefined)
  const [error, setError] = useState<string | null>(null)

  const { data, loading: rolesLoading } = useRolesQuery({
    variables: { first: 100 },
    skip: !open,
  })

  const roles = data?.roles.edges.map((edge) => edge.node) || []

  const [assignRole, { loading: assignLoading }] = useUserUpdateRoleMutation()
  const isLoading = rolesLoading || assignLoading

  useEffect(() => {
    if (open) {
      setSelectedRoleId(undefined)
      setError(null)
    }
  }, [open])

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    const newRoleId = selectedRoleId
    if (newRoleId === currentRoleId || !newRoleId) return

    try {
      await assignRole({
        variables: {
          input: {
            userId: userId,
            roleId: newRoleId,
          },
        },
      })

      toast.success(t("success"))
      onOpenChange(false)
    } catch (error) {
      console.error("Failed to update role:", error)
      setError(t("error"))
    }
  }

  const selectedRole = selectedRoleId
    ? roles.find((role) => role.roleId === selectedRoleId)
    : null

  const permissionSets = selectedRole?.permissionSets || []

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-xl">
        <DialogHeader>
          <DialogTitle>{t("title")}</DialogTitle>
          <DialogDescription>{t("description")}</DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit}>
          <div className="space-y-4">
            <div>
              <Label htmlFor="role">{t("roleLabel")}</Label>
              <Select
                value={selectedRoleId}
                onValueChange={(value) => setSelectedRoleId(value)}
                disabled={isLoading}
              >
                <SelectTrigger>
                  <SelectValue placeholder={t("selectRole")} />
                </SelectTrigger>
                <SelectContent>
                  {roles.map((role) => (
                    <SelectItem key={role.roleId} value={role.roleId}>
                      {role.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <PermissionsDisplay
              permissionSets={permissionSets}
              hasSelectedRole={!!selectedRoleId}
            />

            {error && <p className="text-destructive mt-2">{error}</p>}
          </div>
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
              disabled={isLoading || !selectedRoleId}
            >
              {isLoading ? tCommon("loading") : tCommon("update")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
