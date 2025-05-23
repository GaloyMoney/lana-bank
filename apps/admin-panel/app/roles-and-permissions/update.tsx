"use client"

import { useState, useEffect } from "react"
import { useTranslations } from "next-intl"
import { toast } from "sonner"
import { gql } from "@apollo/client"
import { Edit2 } from "lucide-react"

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@lana/web/ui/dialog"
import { Button } from "@lana/web/ui/button"
import { Input } from "@lana/web/ui/input"
import { Label } from "@lana/web/ui/label"
import { Checkbox } from "@lana/web/ui/check-box"
import { ScrollArea } from "@lana/web/ui/scroll-area"
import { Badge } from "@lana/web/ui/badge"

import {
  usePermissionSetsQuery,
  useRoleAddPermissionSetsMutation,
  useRoleRemovePermissionSetMutation,
} from "@/lib/graphql/generated"
import { useModalNavigation } from "@/hooks/use-modal-navigation"

gql`
  mutation RoleAddPermissionSets($input: RoleAddPermissionSetsInput!) {
    roleAddPermissionSets(input: $input) {
      role {
        ...RoleEntityFields
      }
    }
  }

  mutation RoleRemovePermissionSet($input: RoleRemovePermissionSetInput!) {
    roleRemovePermissionSet(input: $input) {
      role {
        ...RoleEntityFields
      }
    }
  }
`

type UpdateRoleDialogProps = {
  open: boolean
  onOpenChange: (isOpen: boolean) => void
  role: {
    roleId: string
    name: string
    permissionSets: Array<{
      permissionSetId: string
      name: string
    }>
  } | null
}

type DialogMode = "view" | "edit"

export function UpdateRoleDialog({ open, onOpenChange, role }: UpdateRoleDialogProps) {
  const t = useTranslations("RolesAndPermissions.update")
  const tCommon = useTranslations("Common")
  const [mode, setMode] = useState<DialogMode>("view")
  const [name, setName] = useState("")
  const [selectedPermissionSets, setSelectedPermissionSets] = useState<string[]>([])
  const [error, setError] = useState<string | null>(null)

  const { isNavigating } = useModalNavigation({
    closeModal: () => onOpenChange(false),
  })

  useEffect(() => {
    if (role) {
      setName(role.name)
      setSelectedPermissionSets(role.permissionSets.map((ps) => ps.permissionSetId))
      setMode("view")
    }
  }, [role])

  const { data: permissionSetsData, loading: permissionSetsLoading } =
    usePermissionSetsQuery({
      variables: { first: 100 },
    })

  const [addPermissionSets, { loading: addingPermissions }] =
    useRoleAddPermissionSetsMutation({
      update: (cache) => {
        cache.modify({
          fields: {
            roles: (_, { DELETE }) => DELETE,
          },
        })
        cache.gc()
      },
    })

  const [removePermissionSet, { loading: removingPermission }] =
    useRoleRemovePermissionSetMutation({
      update: (cache) => {
        cache.modify({
          fields: {
            roles: (_, { DELETE }) => DELETE,
          },
        })
        cache.gc()
      },
    })

  const permissionSets =
    permissionSetsData?.permissionSets.edges.map((edge) => edge.node) || []
  const isLoading = addingPermissions || removingPermission || isNavigating

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!role) return

    setError(null)

    try {
      const currentPermissionIds = role.permissionSets.map((ps) => ps.permissionSetId)
      const permissionsToAdd = selectedPermissionSets.filter(
        (id) => !currentPermissionIds.includes(id),
      )

      const permissionsToRemove = currentPermissionIds.filter(
        (id) => !selectedPermissionSets.includes(id),
      )

      if (permissionsToAdd.length > 0) {
        await addPermissionSets({
          variables: {
            input: {
              roleId: role.roleId,
              permissionSetIds: permissionsToAdd,
            },
          },
        })
      }

      for (const permissionSetId of permissionsToRemove) {
        await removePermissionSet({
          variables: {
            input: {
              roleId: role.roleId,
              permissionSetId,
            },
          },
        })
      }

      toast.success(t("success"))
      setMode("view")
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

  const handleCancel = () => {
    if (mode === "edit") {
      if (role) {
        setSelectedPermissionSets(role.permissionSets.map((ps) => ps.permissionSetId))
      }
      setError(null)
      setMode("view")
    } else {
      onOpenChange(false)
    }
  }

  const getDialogTitle = () => {
    return mode === "view" ? role?.name || t("viewTitle") : t("editTitle")
  }

  const getDialogDescription = () => {
    return mode === "view" ? t("viewDescription") : t("editDescription")
  }

  return (
    <Dialog
      open={open}
      onOpenChange={(isOpen) => {
        if (!isOpen) {
          setMode("view")
        }
        onOpenChange(isOpen)
      }}
    >
      <DialogContent className="sm:max-w-[600px]">
        <DialogHeader>
          <DialogTitle>{getDialogTitle()}</DialogTitle>
          <DialogDescription>{getDialogDescription()}</DialogDescription>
        </DialogHeader>

        {mode === "view" ? (
          <div>
            <Label>
              {t("permissionsLabel")} ({role?.permissionSets.length || 0} {t("total")})
            </Label>
            <ScrollArea className="h-[250px] py-2">
              {role?.permissionSets.length === 0 ? (
                <div className="text-center text-muted-foreground py-8">
                  {t("noPermissionsAssigned")}
                </div>
              ) : (
                <div className="flex flex-wrap gap-2">
                  {role?.permissionSets.map((permissionSet) => (
                    <Badge key={permissionSet.permissionSetId} variant="secondary">
                      {permissionSet.name}
                    </Badge>
                  ))}
                </div>
              )}
            </ScrollArea>
          </div>
        ) : (
          <form onSubmit={handleSubmit}>
            <div className="space-y-4 py-4">
              <div className="space-y-2">
                <Label htmlFor="name">{t("nameLabel")}</Label>
                <Input id="name" value={name} disabled />
              </div>

              <div className="space-y-2">
                <Label>
                  {t("permissionsLabel")} ({selectedPermissionSets.length} {t("selected")}
                  )
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
                            className="cursor-pointer flex-1"
                          >
                            {permissionSet.name}
                          </Label>
                        </div>
                      ))}
                    </div>
                  )}
                </ScrollArea>
              </div>

              {error && <p className="text-destructive">{error}</p>}
            </div>
          </form>
        )}

        <DialogFooter>
          <div className="flex justify-between w-full">
            <Button variant="outline" onClick={() => onOpenChange(false)}>
              {tCommon("close")}
            </Button>

            <div className="flex gap-2">
              {mode === "view" ? (
                <Button onClick={() => setMode("edit")} className="gap-2">
                  <Edit2 size={16} />
                  {t("editButton")}
                </Button>
              ) : (
                <>
                  <Button variant="outline" onClick={handleCancel} disabled={isLoading}>
                    {tCommon("cancel")}
                  </Button>
                  <Button
                    type="submit"
                    loading={isLoading}
                    disabled={!role || isLoading}
                    onClick={handleSubmit}
                  >
                    {isLoading ? tCommon("loading") : tCommon("save")}
                  </Button>
                </>
              )}
            </div>
          </div>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
