"use client"

import { useState } from "react"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import DateWithTooltip from "@lana/web/components/date-with-tooltip"

import {
  Custodian,
  CustodiansSort,
  SortDirection,
  useCustodiansQuery,
} from "@/lib/graphql/generated"
import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"
import { camelToScreamingSnake } from "@/lib/utils"

gql`
  fragment CustodianFields on Custodian {
    id
    custodianId
    createdAt
    name
    provider
    isManual
  }

  query Custodians($first: Int!, $after: String, $sort: CustodiansSort) {
    custodians(first: $first, after: $after, sort: $sort) {
      edges {
        cursor
        node {
          ...CustodianFields
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

const CustodiansList = () => {
  const t = useTranslations("Custodians.table")
  const [sortBy, setSortBy] = useState<CustodiansSort | null>(null)

  const { data, loading, error, fetchMore } = useCustodiansQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
      sort: sortBy,
    },
  })

  return (
    <div>
      {error && <p className="text-destructive text-sm">{error?.message}</p>}
      <PaginatedTable<Custodian>
        columns={columns(t)}
        data={data?.custodians as PaginatedData<Custodian>}
        loading={loading}
        fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
        pageSize={DEFAULT_PAGESIZE}
        onSort={(column, direction) => {
          setSortBy({
            by: camelToScreamingSnake(column) as CustodiansSort["by"],
            direction: direction as SortDirection,
          })
        }}
      />
    </div>
  )
}

export default CustodiansList

const columns = (t: ReturnType<typeof useTranslations>): Column<Custodian>[] => [
  {
    key: "name",
    label: t("headers.name"),
    sortable: true,
  },
  {
    key: "provider",
    label: t("headers.provider"),
  },
  {
    key: "createdAt",
    label: t("headers.created"),
    render: (createdAt) => <DateWithTooltip value={createdAt} />,
    sortable: true,
  },
]
