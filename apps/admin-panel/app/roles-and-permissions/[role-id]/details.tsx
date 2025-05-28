"use client"

import React from "react"
import { useTranslations } from "next-intl"
import { useRouter } from "next/navigation"

import { Button } from "@lana/web/ui/button"
import { Edit2 } from "lucide-react"
import { Label } from "@lana/web/ui/label"

import { DetailsCard, DetailItemProps } from "@/components/details"
import { RoleQuery } from "@/lib/graphql/generated"
import { formatDate } from "@/lib/utils"
import { usePermissionDisplay } from "@/hooks/use-permission-display"

type RoleDetailsProps = {
  role: NonNullable<RoleQuery["role"]>
}

export const RoleDetailsCard: React.FC<RoleDetailsProps> = ({ role }) => {
  const t = useTranslations("RolesAndPermissions.roleDetails")
  const { getTranslation } = usePermissionDisplay()
  const router = useRouter()

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
                  {getTranslation(permissionSet.name).label}
                </div>
                <p className="text-muted-foreground text-sm">
                  {getTranslation(permissionSet.name).description}
                </p>
              </div>
            ))}
        </div>
      </div>
    </>
  )

  return (
    <DetailsCard
      title={t("roleDetails")}
      description={t("roleDetailsDescription")}
      details={details}
      footerContent={footerContent}
      alignment="left"
      headerAction={
        <Button
          variant="outline"
          onClick={() => router.push(`/roles-and-permissions/${role.roleId}/edit`)}
          className="flex items-center gap-2"
        >
          <Edit2 className="h-4 w-4" />
          {t("editRole")}
        </Button>
      }
      columns={2}
    />
  )
}
