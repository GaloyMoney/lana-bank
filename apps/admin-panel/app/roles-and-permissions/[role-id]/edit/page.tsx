"use client"

import { useState, useEffect, use } from "react"
import { useTranslations } from "next-intl"
import { toast } from "sonner"
import { useRouter } from "next/navigation"
import { gql } from "@apollo/client"

import { Button } from "@lana/web/ui/button"
import { Label } from "@lana/web/ui/label"
import { Checkbox } from "@lana/web/ui/checkbox"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@lana/web/ui/card"

import {
  usePermissionSetsQuery,
  useRoleAddPermissionSetsMutation,
  useRoleRemovePermissionSetsMutation,
  useRoleQuery,
} from "@/lib/graphql/generated"
import { usePermissionDisplay } from "@/hooks/use-permission-display"

gql`
  mutation RoleAddPermissionSets($input: RoleAddPermissionSetsInput!) {
    roleAddPermissionSets(input: $input) {
      role {
        ...RoleFields
      }
    }
  }

  mutation RoleRemovePermissionSets($input: RoleRemovePermissionSetsInput!) {
    roleRemovePermissionSets(input: $input) {
      role {
        ...RoleFields
      }
    }
  }

  query Role($id: UUID!) {
    role(id: $id) {
      ...RoleFields
    }
  }
`

export default function EditRolePage({
  params,
}: {
  params: Promise<{
    "role-id": string
  }>
}) {
  const { "role-id": roleId } = use(params)
  const t = useTranslations("RolesAndPermissions.edit")
  const tCommon = useTranslations("Common")
  const { getTranslation } = usePermissionDisplay()
  const router = useRouter()
  const [selectedPermissionSets, setSelectedPermissionSets] = useState<string[]>([])
  const [error, setError] = useState<string | null>(null)

  const { data: roleData, loading: roleLoading } = useRoleQuery({
    variables: { id: roleId },
  })

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
  const isLoading = roleLoading || addingPermissions || removingPermission

  useEffect(() => {
    if (roleData?.role) {
      setSelectedPermissionSets(
        roleData.role.permissionSets.map((ps) => ps.permissionSetId),
      )
    }
  }, [roleData])

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    try {
      const role = roleData?.role
      if (!role) return

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
        router.push(`/roles-and-permissions/${role.roleId}`)
      }
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

  if (roleLoading) {
    return (
      <Card className="flex flex-col h-[90vh]">
        <CardContent className="flex items-center justify-center h-full">
          {tCommon("loading")}
        </CardContent>
      </Card>
    )
  }

  if (!roleData?.role) {
    return (
      <Card className="flex flex-col h-[90vh]">
        <CardContent className="flex items-center justify-center h-full">
          {tCommon("notFound")}
        </CardContent>
      </Card>
    )
  }

  return (
    <Card className="flex flex-col h-[90vh]">
      <CardHeader>
        <CardTitle>
          {t("title")} {roleData.role.name}
        </CardTitle>
        <CardDescription>{t("editDescription")}</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col flex-1 overflow-hidden">
        <form onSubmit={handleSubmit} className="flex flex-col flex-1 overflow-hidden">
          <div className="flex gap-4 flex-1 overflow-hidden">
            <div className="w-full flex flex-col overflow-hidden">
              <Label className="mb-6">
                {t("permissionsLabel")} ({selectedPermissionSets.length} {t("selected")})
              </Label>
              <div className="permission-list-container flex-1 relative">
                <div className="absolute inset-0 overflow-y-auto pr-2 custom-scrollbar">
                  {permissionSetsLoading ? (
                    <div className="p-4">{tCommon("loading")}</div>
                  ) : permissionSets.length === 0 ? (
                    <div className="p-4">{t("noPermissionsAvailable")}</div>
                  ) : (
                    <div className="space-y-4">
                      {permissionSets.map((permissionSet) => (
                        <div
                          key={permissionSet.permissionSetId}
                          className="border-b pb-4 last:border-b-0"
                        >
                          <div className="flex items-start gap-2">
                            <Checkbox
                              id={permissionSet.permissionSetId}
                              checked={selectedPermissionSets.includes(
                                permissionSet.permissionSetId,
                              )}
                              onCheckedChange={() =>
                                togglePermissionSet(permissionSet.permissionSetId)
                              }
                              disabled={isLoading}
                              className="mt-1"
                            />
                            <div>
                              <Label
                                htmlFor={permissionSet.permissionSetId}
                                className="font-medium"
                              >
                                {getTranslation(permissionSet.name).label}
                              </Label>
                              <p className="text-sm text-muted-foreground">
                                {getTranslation(permissionSet.name).description}
                              </p>
                            </div>
                          </div>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              </div>
              <div className="flex justify-end gap-2 mt-4">
                <Button
                  variant="outline"
                  onClick={() => router.push(`/roles-and-permissions/${roleId}`)}
                  type="button"
                >
                  {tCommon("cancel")}
                </Button>
                <Button type="submit" disabled={isLoading} loading={isLoading}>
                  {tCommon("update")}
                </Button>
              </div>
              {error && <p className="text-destructive mt-4">{error}</p>}
            </div>
          </div>
        </form>
      </CardContent>
    </Card>
  )
}
