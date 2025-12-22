"use client"

import { useState } from "react"
import { useTranslations } from "next-intl"
import { toast } from "sonner"
import { useRouter } from "next/navigation"

import { Button } from "@lana/web/ui/button"
import { Input } from "@lana/web/ui/input"
import { Label } from "@lana/web/ui/label"
import { Checkbox } from "@lana/web/ui/checkbox"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@lana/web/ui/card"
import { Separator } from "@lana/web/ui/separator"

import { gql } from "@apollo/client"

import { usePermissionSetsQuery, useRoleCreateMutation } from "@/lib/graphql/generated"
import { usePermissionDisplay } from "@/hooks/use-permission-display"

gql`
  mutation RoleCreate($input: RoleCreateInput!) {
    roleCreate(input: $input) {
      role {
        ...RoleFields
      }
    }
  }

  query PermissionSets($first: Int!, $after: String) {
    permissionSets(first: $first, after: $after) {
      edges {
        node {
          ...PermissionSetFields
        }
      }
    }
  }
`

export default function CreateRolePage() {
  const t = useTranslations("RolesAndPermissions.create")
  const tCommon = useTranslations("Common")
  const { getTranslation } = usePermissionDisplay()
  const router = useRouter()
  const [name, setName] = useState("")
  const [selectedPermissionSets, setSelectedPermissionSets] = useState<string[]>([])
  const [error, setError] = useState<string | null>(null)

  const { data: permissionSetsData, loading: permissionSetsLoading } =
    usePermissionSetsQuery({
      variables: { first: 100 },
    })

  const [createRole, { loading: creatingRole }] = useRoleCreateMutation()

  const permissionSets =
    permissionSetsData?.permissionSets.edges.map((edge) => edge.node) || []
  const isLoading = creatingRole

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    if (!name || selectedPermissionSets.length === 0) return

    try {
      const result = await createRole({
        variables: {
          input: {
            name,
            permissionSetIds: selectedPermissionSets,
          },
        },
      })

      if (result.data) {
        toast.success(t("success"))
        const newRoleId = result.data.roleCreate.role.roleId
        router.push(`/roles-and-permissions/${newRoleId}`)
      }
    } catch (error) {
      console.error("Failed to create role:", error)
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
    <Card className="flex flex-col h-[90vh]">
      <CardHeader>
        <CardTitle>{t("title")}</CardTitle>
        <CardDescription>{t("description")}</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col flex-1 overflow-hidden">
        <form onSubmit={handleSubmit} className="flex flex-col flex-1 overflow-hidden">
          <div className="flex gap-4 flex-1 overflow-hidden">
            <div className="flex flex-col justify-between w-1/3 p-2">
              <div>
                <Label htmlFor="name">{t("nameLabel")}</Label>
                <Input
                  id="name"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  placeholder={t("namePlaceholder")}
                  disabled={isLoading}
                  className="mt-2"
                />
                {error && <p className="text-destructive mt-4">{error}</p>}
              </div>
              <div className="flex flex-row-reverse gap-2 mt-auto pt-4">
                <Button
                  type="submit"
                  disabled={!name || selectedPermissionSets.length === 0 || isLoading}
                  loading={isLoading}
                >
                  {tCommon("create")}
                </Button>
                <Button variant="outline" onClick={() => router.back()} type="button">
                  {tCommon("cancel")}
                </Button>
              </div>
            </div>

            <div>
              <Separator orientation="vertical" />
            </div>

            <div className="w-2/3 flex flex-col overflow-hidden">
              <Label className="mb-6">
                {t("selectedPermissions", { count: selectedPermissionSets.length })}
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
            </div>
          </div>
        </form>
      </CardContent>
    </Card>
  )
}
