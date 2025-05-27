"use client"

import React, { useState } from "react"
import { useTranslations } from "next-intl"

import { Button } from "@lana/web/ui/button"
import { Edit2 } from "lucide-react"
import { Label } from "@lana/web/ui/label"

import { UpdateRoleDialog } from "../update"

import { DetailsCard, DetailItemProps } from "@/components/details"
import { RoleQuery } from "@/lib/graphql/generated"
import { formatDate } from "@/lib/utils"

type RoleDetailsProps = {
  role: NonNullable<RoleQuery["role"]>
}

export const RoleDetailsCard: React.FC<RoleDetailsProps> = ({ role }) => {
  const t = useTranslations("RolesAndPermissions.roleDetails")
  const permissionT = useTranslations("Permissions")
  const [isUpdateDialogOpen, setIsUpdateDialogOpen] = useState(false)

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
      <div className="w-full flex-1">
        <Label>{t("permissionSets")}</Label>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 gap-x-8 mt-2">
          {[...role.permissionSets]
            .sort((a, b) => a.name.localeCompare(b.name))
            .map((permissionSet) => (
              <div
                key={permissionSet.permissionSetId}
                className="rounded-md py-2 space-y-1"
              >
                <div className="font-medium">
                  {permissionT(`${permissionSet.name}.label`)}
                </div>
                <p className="text-muted-foreground">
                  {permissionT(`${permissionSet.name}.description`)}
                </p>
              </div>
            ))}
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
        />
      )}
    </>
  )
}
