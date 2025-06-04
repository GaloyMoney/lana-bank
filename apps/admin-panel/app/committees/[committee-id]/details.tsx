"use client"

import React from "react"
import { useTranslations } from "next-intl"

import { Button } from "@lana/web/ui/button"

import { formatDate } from "@lana/web/utils"

import { AddUserCommitteeDialog } from "../add-user"

import { DetailsCard, DetailItemProps } from "@/components/details"
import { GetCommitteeDetailsQuery } from "@/lib/graphql/generated"

type CommitteeDetailsProps = {
  committee: NonNullable<GetCommitteeDetailsQuery["committee"]>
}

export const CommitteeDetailsCard: React.FC<CommitteeDetailsProps> = ({ committee }) => {
  const t = useTranslations("Committees.CommitteeDetails.detailsCard")
  const [openAddUserDialog, setOpenAddUserDialog] = React.useState(false)

  const details: DetailItemProps[] = [
    { label: t("fields.createdAt"), value: formatDate(committee.createdAt) },
    { label: t("fields.name"), value: committee.name },
    { label: t("fields.totalMembers"), value: committee.currentMembers.length },
  ]

  const footerContent = (
    <Button
      variant="outline"
      onClick={() => setOpenAddUserDialog(true)}
      data-testid="committee-add-member-button"
    >
      {t("buttons.addMember")}
    </Button>
  )

  return (
    <>
      <DetailsCard
        title={t("title")}
        details={details}
        footerContent={footerContent}
        className="w-full"
      />

      <AddUserCommitteeDialog
        committeeId={committee.committeeId}
        openAddUserDialog={openAddUserDialog}
        setOpenAddUserDialog={setOpenAddUserDialog}
      />
    </>
  )
}

export default CommitteeDetailsCard
