"use client"

import React, { useState } from "react"
import { useTranslations } from "next-intl"

import { Button } from "@lana/web/ui/button"

import Link from "next/link"

import { formatDate } from "@lana/web/utils"

import { UpdateUserRoleDialog } from "../update-role"

import { DetailsCard, DetailItemProps } from "@/components/details"
import { GetUserDetailsQuery } from "@/lib/graphql/generated"

type UserDetailsProps = {
  user: NonNullable<GetUserDetailsQuery["user"]>
}

const UserDetailsCard: React.FC<UserDetailsProps> = ({ user }) => {
  const t = useTranslations("Users.userDetails")
  const [isUpdateRoleDialogOpen, setIsUpdateRoleDialogOpen] = useState(false)

  const handleOpenRoleDialog = () => {
    setIsUpdateRoleDialogOpen(true)
  }

  const details: DetailItemProps[] = [
    { label: t("fields.createdAt"), value: formatDate(user.createdAt) },
    { label: t("fields.email"), value: user.email, valueTestId: "user-details-email" },
    {
      label: t("fields.role"),
      value: (
        <div className="flex flex-wrap gap-2">
          {user.role ? (
            <Link href={`/roles-and-permissions/${user.role.roleId}`}>
              {user.role.name}
            </Link>
          ) : (
            <span className="text-muted-foreground">{t("noRoleAssigned")}</span>
          )}
        </div>
      ),
    },
  ]

  const footer = (
    <Button
      variant="outline"
      onClick={handleOpenRoleDialog}
      data-testid="user-details-manage-role"
    >
      {t("updateRole")}
    </Button>
  )

  return (
    <>
      <DetailsCard
        title={t("title")}
        details={details}
        footerContent={footer}
        columns={3}
      />

      <UpdateUserRoleDialog
        open={isUpdateRoleDialogOpen}
        onOpenChange={setIsUpdateRoleDialogOpen}
        userId={user.userId}
        currentRoleId={user.role?.roleId}
      />
    </>
  )
}

export default UserDetailsCard
