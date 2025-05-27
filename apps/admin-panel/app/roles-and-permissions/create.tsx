"use client"

import { useState } from "react"
import { useTranslations } from "next-intl"
import { toast } from "sonner"
import { useRouter } from "next/navigation"

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

import { gql } from "@apollo/client"

import { usePermissionSetsQuery, useRoleCreateMutation } from "@/lib/graphql/generated"

import { useModalNavigation } from "@/hooks/use-modal-navigation"

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

type CreateRoleDialogProps = {
  open: boolean
  onOpenChange: (isOpen: boolean) => void
}

export function CreateRoleDialog({ open, onOpenChange }: CreateRoleDialogProps) {
  const t = useTranslations("RolesAndPermissions.create")
  const tCommon = useTranslations("Common")
  const permissionT = useTranslations("Permissions")
  const router = useRouter()
  const [name, setName] = useState("")
  const [selectedPermissionSets, setSelectedPermissionSets] = useState<string[]>([])
  const [error, setError] = useState<string | null>(null)

  const { isNavigating } = useModalNavigation({
    closeModal: () => onOpenChange(false),
  })

  const { data: permissionSetsData, loading: permissionSetsLoading } =
    usePermissionSetsQuery({
      variables: { first: 100 },
    })

  const [createRole, { loading: creatingRole }] = useRoleCreateMutation()

  const permissionSets =
    permissionSetsData?.permissionSets.edges.map((edge) => edge.node) || []
  const isLoading = creatingRole || isNavigating

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
        resetForm()
        onOpenChange(false)
        const newRoleId = result.data.roleCreate.role.roleId
        router.push(`/roles-and-permissions/${newRoleId}`)
      }
    } catch (error) {
      console.error("Failed to create role:", error)
      const errorMessage = error instanceof Error ? error.message : "Unknown error"
      setError(t("error", { error: errorMessage }))
    }
  }

  const resetForm = () => {
    setName("")
    setSelectedPermissionSets([])
    setError(null)
  }

  const togglePermissionSet = (permissionSetId: string) => {
    setSelectedPermissionSets((prev) =>
      prev.includes(permissionSetId)
        ? prev.filter((id) => id !== permissionSetId)
        : [...prev, permissionSetId],
    )
  }

  return (
    <Dialog
      open={open}
      onOpenChange={(isOpen) => {
        onOpenChange(isOpen)
        if (!isOpen) resetForm()
      }}
    >
      <DialogContent className="sm:max-w-[600px]">
        <DialogHeader>
          <DialogTitle>{t("title")}</DialogTitle>
          <DialogDescription>{t("description")}</DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit}>
          <div className="flex flex-col gap-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="name">{t("nameLabel")}</Label>
              <Input
                id="name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder={t("namePlaceholder")}
                disabled={isLoading}
              />
            </div>

            <div className="space-y-2">
              <Label>{t("permissionsLabel")}</Label>
              <div className="border rounded-md">
                <ScrollArea className="h-[300px] p-2">
                  {permissionSetsLoading ? (
                    <div className="p-2">{tCommon("loading")}</div>
                  ) : permissionSets.length === 0 ? (
                    <div className="p-2">{t("noPermissionsSelected")}</div>
                  ) : (
                    <div className="space-y-2">
                      {permissionSets.map((permissionSet) => (
                        <div
                          key={permissionSet.permissionSetId}
                          className="flex items-start space-x-2 p-2 hover:bg-accent rounded-md"
                        >
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
                          <div className="space-y-1">
                            <Label
                              htmlFor={permissionSet.permissionSetId}
                              className="text-sm font-medium cursor-pointer"
                            >
                              {permissionT(`${permissionSet.name}.label`)}
                            </Label>
                            <p className="text-sm text-muted-foreground">
                              {permissionT(`${permissionSet.name}.description`)}
                            </p>
                          </div>
                        </div>
                      ))}
                    </div>
                  )}
                </ScrollArea>
              </div>
            </div>

            {error && <p className="text-destructive">{error}</p>}
          </div>

          <DialogFooter>
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
              disabled={!name || selectedPermissionSets.length === 0 || isLoading}
            >
              {isLoading ? tCommon("loading") : tCommon("create")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
