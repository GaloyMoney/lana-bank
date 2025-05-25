"use client"

import React, { useState, useCallback } from "react"
import { useTranslations } from "next-intl"
import { useRouter } from "next/navigation"

import { Badge } from "@lana/web/ui/badge"
import { Label } from "@lana/web/ui/label"
import { Button } from "@lana/web/ui/button"
import { Edit2 } from "lucide-react"

import { UpdateRoleDialog } from "../update"

import { DetailsCard, DetailItemProps } from "@/components/details"
import { RoleQuery } from "@/lib/graphql/generated"

import { formatDate } from "@/lib/utils"

type RoleDetailsProps = {
  role: NonNullable<RoleQuery["role"]>
}

export const RoleDetailsCard: React.FC<RoleDetailsProps> = ({ role }) => {
  const t = useTranslations("RolesAndPermissions.roleDetails")
  const [isUpdateDialogOpen, setIsUpdateDialogOpen] = useState(false)
  const router = useRouter()

  const handleRoleUpdated = useCallback(() => {
    // Refresh the current route data without a full page reload
    router.refresh()
  }, [router])

  const details: DetailItemProps[] = [
    {
      label: t("name"),
      value: role.name,
    },

    {
      label: t("createdAt"),
      value: formatDate(role.createdAt),
    },
  ]

  const footerContent = (
    <>
      <div>
        <Label>{t("permissionSets")}</Label>
        <div className="flex flex-wrap gap-2 items-center py-2">
          {role.permissionSets
            .map((permissionSet) => (
              <Badge
                variant="outline"
                className="text-sm"
                key={permissionSet.permissionSetId}
              >
                {permissionSet.name}
              </Badge>
            ))
            .sort((a, b) => a.props.children.localeCompare(b.props.children))}
        </div>
      </div>
    </>
  )

  return (
    <>
      <DetailsCard
        title={t("roleDetails")}
        description={t("roleDetailsDescription")}
        details={details}
        footerContent={footerContent}
        alignment="left"
        headerAction={
          <Button
            variant="outline"
            onClick={() => setIsUpdateDialogOpen(true)}
            className="flex items-center gap-2"
          >
            <Edit2 className="h-4 w-4" />
            {t("editRole")}
          </Button>
        }
        columns={2}
      />
      {isUpdateDialogOpen && (
        <UpdateRoleDialog
          open={isUpdateDialogOpen}
          onOpenChange={setIsUpdateDialogOpen}
          role={role}
          onRoleUpdated={handleRoleUpdated}
        />
      )}
    </>
  )
}
