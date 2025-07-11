"use client"

import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import DateWithTooltip from "@lana/web/components/date-with-tooltip"

import { Custodian, useCustodiansQuery } from "@/lib/graphql/generated"
import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"

gql`
  fragment CustodianFields on Custodian {
    id
    custodianId
    createdAt
    name
  }

  query Custodians($first: Int!, $after: String) {
    custodians(first: $first, after: $after) {
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

  const { data, loading, error, fetchMore } = useCustodiansQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
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
      />
    </div>
  )
}

export default CustodiansList

const columns = (t: ReturnType<typeof useTranslations>): Column<Custodian>[] => [
  {
    key: "name",
    label: t("headers.name"),
  },
  {
    key: "createdAt",
    label: t("headers.created"),
    render: (createdAt) => <DateWithTooltip value={createdAt} />,
  },
]
