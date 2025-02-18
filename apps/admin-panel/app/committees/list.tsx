"use client"

import { gql } from "@apollo/client"
import { useState } from "react"

import { CreateCommitteeDialog } from "./create"
import { AddUserCommitteeDialog } from "./add-user"

import { Committee, useCommitteesQuery } from "@/lib/graphql/generated"
import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"
import { formatDate } from "@/lib/utils"

gql`
  fragment CommitteeFields on Committee {
    id
    committeeId
    createdAt
    name
    currentMembers {
      id
      userId
      email
      roles
    }
  }

  query Committees($first: Int!, $after: String) {
    committees(first: $first, after: $after) {
      edges {
        cursor
        node {
          ...CommitteeFields
        }
      }
      pageInfo {
        endCursor
        startCursor
        hasNextPage
        hasPreviousPage
      }
    }
  }
`

const CommitteesList = () => {
  const [openCreateCommitteeDialog, setOpenCreateCommitteeDialog] =
    useState<boolean>(false)
  const [openAddUserDialog, setOpenAddUserDialog] = useState<Committee | null>(null)

  const { data, loading, error, fetchMore } = useCommitteesQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
    },
  })

  return (
    <div>
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

      {error && <p className="text-destructive text-sm">{error?.message}</p>}
      <PaginatedTable<Committee>
        columns={columns}
        data={data?.committees as PaginatedData<Committee>}
        loading={loading}
        fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
        pageSize={DEFAULT_PAGESIZE}
        navigateTo={(committee) => `/committees/${committee.committeeId}`}
      />
    </div>
  )
}

export default CommitteesList

const columns: Column<Committee>[] = [
  {
    key: "name",
    label: "Name",
  },
  {
    key: "createdAt",
    label: "Created",
    render: (createdAt) => formatDate(createdAt, { includeTime: false }),
  },
  {
    key: "currentMembers",
    label: "Members",
    render: (currentMembers) => currentMembers.length,
  },
]
