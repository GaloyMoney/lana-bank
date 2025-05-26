"use client"

import { gql } from "@apollo/client"
import { useState } from "react"
import { useTranslations } from "next-intl"

import { CreateCommitteeDialog } from "./create"
import { AddUserCommitteeDialog } from "./add-user"

import { Committee, useCommitteesQuery } from "@/lib/graphql/generated"
import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"
import DateWithTooltip from "@/components/date-with-tooltip"

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
  const t = useTranslations("Committees.table")
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
        columns={columns(t)}
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

const columns = (t: ReturnType<typeof useTranslations>): Column<Committee>[] => [
  {
    key: "name",
    label: t("headers.name"),
  },
  {
    key: "createdAt",
    label: t("headers.created"),
    render: (createdAt) => <DateWithTooltip value={createdAt} />,
  },
  {
    key: "currentMembers",
    label: t("headers.members"),
    render: (currentMembers) => currentMembers.length,
  },
]
