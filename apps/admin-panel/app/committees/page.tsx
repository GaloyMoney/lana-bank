"use client"
import React, { useState } from "react"
import { gql } from "@apollo/client"

import { useRouter } from "next/navigation"

import { CreateCommitteeDialog } from "./create"
import { AddUserCommitteeDialog } from "./add-user"

import { PageHeading } from "@/components/page-heading"
import { Button } from "@/components/primitive/button"
import { Committee, useMeQuery, useCommitteesQuery } from "@/lib/graphql/generated"
import { Card, CardContent } from "@/components/primitive/card"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/primitive/table"
import { formatDate } from "@/lib/utils"

gql`
  query Committees($first: Int!, $after: String) {
    committees(first: $first, after: $after) {
      edges {
        cursor
        node {
          id
          committeeId
          createdAt
          name
          currentMembers {
            userId
          }
        }
      }
      pageInfo {
        endCursor
        hasNextPage
      }
    }
  }
`

function CommitteesPage() {
  const [openCreateCommitteeDialog, setOpenCreateCommitteeDialog] =
    useState<boolean>(false)
  const [openAddUserDialog, setOpenAddUserDialog] = useState<Committee | null>(null)

  const router = useRouter()
  const { data: me } = useMeQuery()
  const { data, loading, error, fetchMore } = useCommitteesQuery({
    variables: {
      first: 20,
    },
    fetchPolicy: "cache-and-network",
  })

  if (loading && !data) {
    return (
      <main>
        <div className="flex justify-between items-center mb-8">
          <PageHeading className="mb-0">Committees</PageHeading>
        </div>
        <Card>
          <CardContent>
            <p className="mt-6">Loading...</p>
          </CardContent>
        </Card>
      </main>
    )
  }

  if (error) {
    return (
      <main>
        <div className="flex justify-between items-center mb-8">
          <PageHeading className="mb-0">Committees</PageHeading>
        </div>
        <Card>
          <CardContent>
            <p className="text-destructive mt-6">{error.message}</p>
          </CardContent>
        </Card>
      </main>
    )
  }

  return (
    <main>
      {openAddUserDialog && (
        <AddUserCommitteeDialog
          committeeId={openAddUserDialog.committeeId}
          openAddUserDialog={Boolean(openAddUserDialog)}
          setOpenAddUserDialog={() => setOpenAddUserDialog(null)}
        />
      )}
      <CreateCommitteeDialog
        openCreateCommitteeDialog={openCreateCommitteeDialog}
        setOpenCreateCommitteeDialog={setOpenCreateCommitteeDialog}
      />

      <div className="flex justify-between items-center mb-8">
        <PageHeading className="mb-0">Committees</PageHeading>
        {me?.me.subjectCanCreateUser && (
          <Button onClick={() => setOpenCreateCommitteeDialog(true)}>Create New</Button>
        )}
      </div>

      <Card>
        <CardContent>
          {!data?.committees.edges || data.committees.edges.length === 0 ? (
            <p className="mt-6">No committees found</p>
          ) : (
            <Table className="mt-6">
              <TableHeader>
                <TableRow>
                  <TableHead>Name</TableHead>
                  <TableHead>Created</TableHead>
                  <TableHead>Members</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {data.committees.edges.map(({ node: committee }) => (
                  <TableRow
                    key={committee.committeeId}
                    className="cursor-pointer"
                    onClick={() => router.push(`/committees/${committee.committeeId}`)}
                  >
                    <TableCell>{committee.name}</TableCell>
                    <TableCell>{formatDate(committee.createdAt)}</TableCell>
                    <TableCell>{committee.currentMembers.length}</TableCell>
                  </TableRow>
                ))}
                {data.committees.pageInfo.hasNextPage && (
                  <TableRow
                    className="cursor-pointer"
                    onClick={() =>
                      fetchMore({
                        variables: {
                          after:
                            data.committees.edges[data.committees.edges.length - 1]
                              .cursor,
                        },
                      })
                    }
                  >
                    <TableCell>
                      <div className="font-thin italic">show more...</div>
                    </TableCell>
                    <TableCell />
                    <TableCell />
                  </TableRow>
                )}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>
    </main>
  )
}

export default CommitteesPage
