"use client"

import React, { useState } from "react"
import Link from "next/link"

import { IoTrashOutline } from "react-icons/io5"

import { RemoveUserCommitteeDialog } from "../remove-user"

import { GetCommitteeDetailsQuery } from "@/lib/graphql/generated"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/primitive/card"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/primitive/table"
import { Button } from "@/components/primitive/button"
import { formatRole } from "@/lib/utils"
import { Badge } from "@/components/primitive/badge"

type CommitteeUsersProps = {
  committee: NonNullable<GetCommitteeDetailsQuery["committee"]>
}

type UserToRemove = {
  userId: string
  email: string
} | null

export const CommitteeUsers: React.FC<CommitteeUsersProps> = ({ committee }) => {
  const [userToRemove, setUserToRemove] = useState<UserToRemove>(null)

  return (
    <>
      {committee.currentMembers.length > 0 ? (
        <Card>
          <CardHeader>
            <CardTitle>Committee Members</CardTitle>
          </CardHeader>
          <CardContent>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Email</TableHead>
                  <TableHead>User ID</TableHead>
                  <TableHead></TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {committee.currentMembers.map((user) => (
                  <TableRow key={user.userId}>
                    <TableCell>
                      <Link href={`/users/${user.userId}`}>{user.email}</Link>
                    </TableCell>
                    <TableCell>{user.userId}</TableCell>
                    <TableCell>
                      <div className="flex flex-wrap gap-2 text-textColor-secondary items-center">
                        {user.roles.length > 0
                          ? user.roles.sort().map((role) => (
                              <Badge variant="secondary" key={role}>
                                {formatRole(role)}
                              </Badge>
                            ))
                          : "No roles Assigned"}
                      </div>
                    </TableCell>

                    <TableCell className="text-right px-1">
                      <Button
                        className="gap-2 text-destructive"
                        variant="ghost"
                        onClick={() =>
                          setUserToRemove({
                            userId: user.userId,
                            email: user.email,
                          })
                        }
                      >
                        <IoTrashOutline className="w-4 h-4" />
                        Remove member
                      </Button>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </CardContent>
        </Card>
      ) : (
        <Card>
          <CardContent>
            <p className="mt-6">No members found in this committee</p>
          </CardContent>
        </Card>
      )}
      {userToRemove && (
        <RemoveUserCommitteeDialog
          committeeId={committee.committeeId}
          userId={userToRemove.userId}
          userEmail={userToRemove.email}
          openRemoveUserDialog={Boolean(userToRemove)}
          setOpenRemoveUserDialog={() => setUserToRemove(null)}
        />
      )}
    </>
  )
}
