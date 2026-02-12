import React from "react"
import { useTranslations } from "next-intl"
import { Label } from "@lana/web/ui/label"
import { ScrollArea } from "@lana/web/ui/scroll-area"

import { usePermissionDisplay } from "@/hooks/use-permission-display"

type Permission = {
  name: string
}

type PermissionsDisplayProps = {
  permissionSets: Permission[]
  hasSelectedRole: boolean
  className?: string
}

export function PermissionsDisplay({
  permissionSets,
  hasSelectedRole,
  className,
}: PermissionsDisplayProps) {
  const t = useTranslations("PermissionsDisplay")
  const { getTranslation } = usePermissionDisplay()

  return (
    <div className={className}>
      <Label className="mb-2 block">{t("permissionsLabel")}</Label>
      <ScrollArea className="h-[300px] rounded-md border p-4">
        {permissionSets.length > 0 ? (
          <div className="space-y-4">
            {permissionSets.map((permission) => {
              const { label, description } = getTranslation(permission.name)
              return (
                <div key={permission.name} className="space-y-1">
                  <p className="text-sm font-medium">{label}</p>
                  <p className="text-sm text-muted-foreground">{description}</p>
                </div>
              )
            })}
          </div>
        ) : (
          <div className="flex h-full w-full items-center justify-center">
            <p className="text-center text-muted-foreground">
              {hasSelectedRole ? t("noPermissions") : t("noRoleSelected")}
            </p>
          </div>
        )}
      </ScrollArea>
    </div>
  )
}
