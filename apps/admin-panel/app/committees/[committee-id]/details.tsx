"use client"

import React from "react"

import { AddUserCommitteeDialog } from "../add-user"

import { GetCommitteeDetailsQuery } from "@/lib/graphql/generated"
import { DetailItem, DetailsGroup } from "@/components/details"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/primitive/card"
import { Button } from "@/components/primitive/button"
import { formatDate } from "@/lib/utils"

type CommitteeDetailsProps = {
  committee: NonNullable<GetCommitteeDetailsQuery["committee"]>
}

export const CommitteeDetailsCard: React.FC<CommitteeDetailsProps> = ({ committee }) => {
  const [openAddUserDialog, setOpenAddUserDialog] = React.useState(false)

  return (
    <div className="flex">
      <Card className="w-full">
        <CardHeader className="flex-row justify-between items-center">
          <CardTitle>Committee</CardTitle>
        </CardHeader>
        <CardContent>
          <DetailsGroup>
            <DetailItem label="Name" value={committee.name} />
            <DetailItem label="Created At" value={formatDate(committee.createdAt)} />
            <DetailItem label="Total Members" value={committee.currentMembers.length} />
          </DetailsGroup>
        </CardContent>
      </Card>

      <div className="flex flex-col space-y-2 mt-1 ml-4">
        <>
          <Button
            variant="primary"
            className="w-full"
            onClick={() => setOpenAddUserDialog(true)}
          >
            Add Member
          </Button>
        </>
      </div>

      <AddUserCommitteeDialog
        committeeId={committee.committeeId}
        openAddUserDialog={openAddUserDialog}
        setOpenAddUserDialog={setOpenAddUserDialog}
      />
    </div>
  )
}
