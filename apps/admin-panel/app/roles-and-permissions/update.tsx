"use client"

import { useState, useEffect } from "react"
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
import { Label } from "@lana/web/ui/label"
import { Checkbox } from "@lana/web/ui/check-box"
import { ScrollArea } from "@lana/web/ui/scroll-area"

import {
  usePermissionSetsQuery,
  useRoleAddPermissionSetsMutation,
  useRoleRemovePermissionSetsMutation,
  RoleQuery,
} from "@/lib/graphql/generated"

gql`
  mutation RoleAddPermissionSets($input: RoleAddPermissionSetsInput!) {
    roleAddPermissionSets(input: $input) {
      role {
        ...RoleEntityFields
      }
    }
  }

  mutation RoleRemovePermissionSets($input: RoleRemovePermissionSetsInput!) {
    roleRemovePermissionSets(input: $input) {
      role {
        ...RoleEntityFields
      }
    }
  }
`

type UpdateRoleDialogProps = {
  open: boolean
  onOpenChange: (isOpen: boolean) => void
  role: NonNullable<RoleQuery["role"]>
  onRoleUpdated?: () => void
}

export function UpdateRoleDialog({
  open,
  onOpenChange,
  role,
  onRoleUpdated,
}: UpdateRoleDialogProps) {
  const t = useTranslations("RolesAndPermissions.update")
  const tCommon = useTranslations("Common")
  const [selectedPermissionSets, setSelectedPermissionSets] = useState<string[]>([])
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (role && open) {
      setSelectedPermissionSets(role.permissionSets.map((ps) => ps.permissionSetId))
    }
  }, [role, open])

  const { data: permissionSetsData, loading: permissionSetsLoading } =
    usePermissionSetsQuery({
      variables: { first: 100 },
    })

  const [addPermissionSets, { loading: addingPermissions }] =
    useRoleAddPermissionSetsMutation()

  const [removePermissionSet, { loading: removingPermission }] =
    useRoleRemovePermissionSetsMutation()

  const permissionSets =
    permissionSetsData?.permissionSets.edges.map((edge) => edge.node) || []
  const isLoading = addingPermissions || removingPermission

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    try {
      const currentPermissionIds = role.permissionSets.map((ps) => ps.permissionSetId)
      const permissionsToAdd = selectedPermissionSets.filter(
        (id) => !currentPermissionIds.includes(id),
      )

      const permissionsToRemove = currentPermissionIds.filter(
        (id) => !selectedPermissionSets.includes(id),
      )

      let mutationsPerformed = false
      if (permissionsToAdd.length > 0) {
        await addPermissionSets({
          variables: {
            input: {
              roleId: role.roleId,
              permissionSetIds: permissionsToAdd,
            },
          },
        })
        mutationsPerformed = true
      }

      if (permissionsToRemove.length > 0) {
        await removePermissionSet({
          variables: {
            input: {
              roleId: role.roleId,
              permissionSetIds: permissionsToRemove,
            },
          },
        })
        mutationsPerformed = true
      }

      if (mutationsPerformed) {
        toast.success(t("success"))
        if (onRoleUpdated) {
          onRoleUpdated()
        }
      }

      onOpenChange(false)
    } catch (error) {
      console.error("Failed to update role:", error)
      const errorMessage = error instanceof Error ? error.message : "Unknown error"
      setError(t("error", { error: errorMessage }))
    }
  }

  const togglePermissionSet = (permissionSetId: string) => {
    setSelectedPermissionSets((prev) =>
      prev.includes(permissionSetId)
        ? prev.filter((id) => id !== permissionSetId)
        : [...prev, permissionSetId],
    )
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[600px]">
        <DialogHeader>
          <DialogTitle>{t("title")}</DialogTitle>
          <DialogDescription>{t("editDescription")}</DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit}>
          <Label>
            {t("permissionsLabel")} ({selectedPermissionSets.length} {t("selected")})
          </Label>
          <ScrollArea className="h-[250px] border rounded-md p-2">
            {permissionSetsLoading ? (
              <div className="p-2">{tCommon("loading")}</div>
            ) : permissionSets.length === 0 ? (
              <div className="p-2">{t("noPermissionsAvailable")}</div>
            ) : (
              <div className="space-y-2">
                {permissionSets.map((permissionSet) => (
                  <div
                    key={permissionSet.permissionSetId}
                    className="flex items-center space-x-2 p-2 hover:bg-accent rounded"
                  >
                    <Checkbox
                      id={`update-${permissionSet.permissionSetId}`}
                      checked={selectedPermissionSets.includes(
                        permissionSet.permissionSetId,
                      )}
                      onCheckedChange={() =>
                        togglePermissionSet(permissionSet.permissionSetId)
                      }
                      disabled={isLoading}
                    />
                    <Label
                      htmlFor={`update-${permissionSet.permissionSetId}`}
                      className="text-sm font-normal cursor-pointer"
                    >
                      {permissionSet.name}
                    </Label>
                  </div>
                ))}
              </div>
            )}
          </ScrollArea>

          {error && <p className="text-destructive mt-4">{error}</p>}

          <DialogFooter className="mt-4">
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
              disabled={isLoading}
            >
              {tCommon("cancel")}
            </Button>
            <Button type="submit" loading={isLoading} disabled={isLoading}>
              {isLoading ? tCommon("loading") : tCommon("update")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
