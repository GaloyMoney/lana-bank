"use client"

import { gql } from "@apollo/client"
import { useState } from "react"
import { useTranslations } from "next-intl"

import DateWithTooltip from "@lana/web/components/date-with-tooltip"

import { CreateCommitteeDialog } from "./create"
import { AddUserCommitteeDialog } from "./add-user"

import {
  Committee,
  CommitteesSort,
  SortDirection,
  useCommitteesQuery,
} from "@/lib/graphql/generated"
import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"
import { camelToScreamingSnake } from "@/lib/utils"

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
      role {
        ...RoleFields
      }
    }
  }

  query Committees($first: Int!, $after: String, $sort: CommitteesSort) {
    committees(first: $first, after: $after, sort: $sort) {
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
  const t = useTranslations("Committees.table")
  const [openCreateCommitteeDialog, setOpenCreateCommitteeDialog] =
    useState<boolean>(false)
  const [openAddUserDialog, setOpenAddUserDialog] = useState<Committee | null>(null)
  const [sortBy, setSortBy] = useState<CommitteesSort | null>(null)

  const { data, loading, error, fetchMore } = useCommitteesQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
      sort: sortBy,
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
        columns={columns(t)}
        data={data?.committees as PaginatedData<Committee>}
        loading={loading}
        fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
        pageSize={DEFAULT_PAGESIZE}
        navigateTo={(committee) => `/committees/${committee.committeeId}`}
        onSort={(column, direction) => {
          setSortBy({
            by: camelToScreamingSnake(column) as CommitteesSort["by"],
            direction: direction as SortDirection,
          })
        }}
      />
    </div>
  )
}

export default CommitteesList

const columns = (t: ReturnType<typeof useTranslations>): Column<Committee>[] => [
  {
    key: "name",
    label: t("headers.name"),
    sortable: true,
  },
  {
    key: "createdAt",
    label: t("headers.created"),
    render: (createdAt) => <DateWithTooltip value={createdAt} />,
    sortable: true,
  },
  {
    key: "currentMembers",
    label: t("headers.members"),
    render: (currentMembers) => currentMembers.length,
  },
]
