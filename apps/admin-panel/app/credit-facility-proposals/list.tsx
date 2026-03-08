"use client"

import { useState } from "react"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import DateWithTooltip from "@lana/web/components/date-with-tooltip"

import { CreditFacilityProposalStatusBadge } from "./status-badge"

import {
  CreditFacilityProposal,
  CreditFacilityProposalStatus,
  CreditFacilityProposalsFilter,
  CreditFacilityProposalsSort,
  SortDirection,
  useCreditFacilityProposalsQuery,
} from "@/lib/graphql/generated"

import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"
import Balance from "@/components/balance/balance"
import { camelToScreamingSnake } from "@/lib/utils"

gql`
  query CreditFacilityProposals($first: Int!, $after: String, $sort: CreditFacilityProposalsSort, $filter: CreditFacilityProposalsFilter) {
    creditFacilityProposals(first: $first, after: $after, sort: $sort, filter: $filter) {
      edges {
        cursor
        node {
          id
          creditFacilityProposalId
          createdAt
          facilityAmount
          status
          customer {
            customerId
            email
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

const CreditFacilityProposals = () => {
  const t = useTranslations("CreditFacilityProposals")
  const commonT = useTranslations("Common")
  const [sortBy, setSortBy] = useState<CreditFacilityProposalsSort | null>(null)
  const [filter, setFilter] = useState<CreditFacilityProposalsFilter | null>(null)

  const { data, loading, error, fetchMore } = useCreditFacilityProposalsQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
      sort: sortBy,
      filter,
    },
  })

  return (
    <div>
      {error && <p className="text-destructive text-sm">{commonT("error")}</p>}
      <PaginatedTable<CreditFacilityProposal>
        columns={columns(t)}
        data={data?.creditFacilityProposals as PaginatedData<CreditFacilityProposal>}
        loading={loading}
        fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
        pageSize={DEFAULT_PAGESIZE}
        navigateTo={(proposal) =>
          `/credit-facility-proposals/${proposal.creditFacilityProposalId}`
        }
        onSort={(column, direction) => {
          setSortBy({
            by: camelToScreamingSnake(column) as CreditFacilityProposalsSort["by"],
            direction: direction as SortDirection,
          })
        }}
        onFilter={(filters) => {
          const f = filters as CreditFacilityProposalsFilter
          setFilter(Object.keys(f).length > 0 ? f : null)
        }}
      />
    </div>
  )
}

export default CreditFacilityProposals

const columns = (t: (key: string) => string): Column<CreditFacilityProposal>[] => [
  {
    key: "status",
    label: t("table.headers.status"),
    labelClassName: "w-[20%]",
    render: (status) => <CreditFacilityProposalStatusBadge status={status} />,
    filterValues: Object.values(CreditFacilityProposalStatus),
    filterLabel: (status) => <CreditFacilityProposalStatusBadge status={status} plain />,
  },
  {
    key: "customer",
    label: t("table.headers.customer"),
    labelClassName: "w-[40%]",
    render: (customer) => <div className="truncate">{customer.email}</div>,
  },
  {
    key: "facilityAmount",
    label: t("table.headers.facilityAmount"),
    labelClassName: "w-[25%]",
    render: (amount) => <Balance amount={amount} currency="usd" />,
    sortable: true,
  },
  {
    key: "createdAt",
    label: t("table.headers.createdAt"),
    labelClassName: "w-[15%]",
    render: (date) => <DateWithTooltip value={date} />,
    sortable: true,
  },
]
