"use client"

import { useState } from "react"

import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"
import { formatDate } from "@lana/web/utils"

import { CustomerTypeBadge } from "../customers/customer-type-badge"

import { ProspectStageBadge } from "./prospect-stage-badge"

import {
  CustomerType,
  Prospect,
  ProspectStage,
  ProspectsFilter,
  ProspectsSort,
  SortDirection,
  useProspectsQuery,
} from "@/lib/graphql/generated"

import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"
import { camelToScreamingSnake } from "@/lib/utils"

gql`
  query Prospects(
    $first: Int!
    $after: String
    $sort: ProspectsSort
    $filter: ProspectsFilter
  ) {
    prospects(first: $first, after: $after, sort: $sort, filter: $filter) {
      edges {
        node {
          id
          prospectId
          publicId
          stage
          level
          email
          telegramHandle
          applicantId
          customerType
          createdAt
        }
        cursor
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

const ProspectsList = () => {
  const t = useTranslations("Prospects")
  const [sortBy, setSortBy] = useState<ProspectsSort | null>(null)
  const [filter, setFilter] = useState<ProspectsFilter | null>(null)

  const { data, loading, error, fetchMore } = useProspectsQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
      sort: sortBy,
      filter,
    },
  })

  const columns: Column<Prospect>[] = [
    {
      key: "email",
      label: t("columns.email"),
      labelClassName: "w-[25%]",
      sortable: true,
    },
    {
      key: "telegramHandle",
      label: t("columns.telegramHandle"),
      labelClassName: "w-[25%]",
      sortable: true,
    },
    {
      key: "stage",
      label: t("columns.stage"),
      filterValues: Object.values(ProspectStage),
      filterLabel: (stage) => <ProspectStageBadge stage={stage} plain />,
      render: (stage) => <ProspectStageBadge stage={stage} />,
    },
    {
      key: "customerType",
      label: t("columns.customerType"),
      filterValues: Object.values(CustomerType),
      filterLabel: (type) => <CustomerTypeBadge customerType={type} />,
    },
    {
      key: "createdAt",
      label: t("columns.createdAt"),
      sortable: true,
      render: (createdAt) => formatDate(createdAt),
    },
  ]

  return (
    <div>
      {error && <p className="text-destructive text-sm">{error?.message}</p>}
      <PaginatedTable<Prospect>
        columns={columns}
        data={data?.prospects as PaginatedData<Prospect>}
        loading={loading}
        fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
        pageSize={DEFAULT_PAGESIZE}
        navigateTo={(prospect) => `/prospects/${prospect.publicId}`}
        onSort={(column, direction) => {
          setSortBy({
            by: camelToScreamingSnake(column) as ProspectsSort["by"],
            direction: direction as SortDirection,
          })
        }}
        onFilter={(filters) => {
          const f = filters as ProspectsFilter
          setFilter(Object.keys(f).length > 0 ? f : null)
        }}
      />
    </div>
  )
}

export default ProspectsList
