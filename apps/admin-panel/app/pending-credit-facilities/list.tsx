"use client"

import { useState } from "react"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import DateWithTooltip from "@lana/web/components/date-with-tooltip"

import { PendingCreditFacilityStatusBadge } from "./status-badge"
import { PendingFacilityCollateralizationStateLabel } from "./label"

import {
  PendingCreditFacility,
  PendingCreditFacilityStatus,
  PendingCreditFacilityCollateralizationState,
  PendingCreditFacilitiesFilter,
  PendingCreditFacilitiesSort,
  SortDirection,
  usePendingCreditFacilitiesQuery,
} from "@/lib/graphql/generated"

import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"
import Balance from "@/components/balance/balance"
import { camelToScreamingSnake } from "@/lib/utils"

gql`
  query PendingCreditFacilities($first: Int!, $after: String, $sort: PendingCreditFacilitiesSort, $filter: PendingCreditFacilitiesFilter) {
    pendingCreditFacilities(first: $first, after: $after, sort: $sort, filter: $filter) {
      edges {
        cursor
        node {
          id
          pendingCreditFacilityId
          createdAt
          collateralizationState
          facilityAmount
          status
          collateral {
            btcBalance
          }
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

const PendingCreditFacilities = () => {
  const t = useTranslations("PendingCreditFacilities")
  const [sortBy, setSortBy] = useState<PendingCreditFacilitiesSort | null>(null)
  const [filter, setFilter] = useState<PendingCreditFacilitiesFilter | null>(null)

  const { data, loading, error, fetchMore } = usePendingCreditFacilitiesQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
      sort: sortBy,
      filter,
    },
  })

  return (
    <div>
      {error && <p className="text-destructive text-sm">{error.message}</p>}
      <PaginatedTable<PendingCreditFacility>
        columns={columns(t)}
        data={data?.pendingCreditFacilities as PaginatedData<PendingCreditFacility>}
        loading={loading}
        fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
        pageSize={DEFAULT_PAGESIZE}
        navigateTo={(pending) =>
          `/pending-credit-facilities/${pending.pendingCreditFacilityId}`
        }
        onSort={(column, direction) => {
          setSortBy({
            by: camelToScreamingSnake(column) as PendingCreditFacilitiesSort["by"],
            direction: direction as SortDirection,
          })
        }}
        onFilter={(filters) => {
          const f = filters as PendingCreditFacilitiesFilter
          setFilter(Object.keys(f).length > 0 ? f : null)
        }}
      />
    </div>
  )
}

export default PendingCreditFacilities

const columns = (
  t: (key: string) => string,
): Column<PendingCreditFacility>[] => [
  {
    key: "status",
    label: t("table.headers.status"),
    labelClassName: "w-[20%]",
    render: (status) => <PendingCreditFacilityStatusBadge status={status} />,
    filterValues: Object.values(PendingCreditFacilityStatus),
    filterLabel: (status) => (
      <PendingCreditFacilityStatusBadge status={status} plain />
    ),
  },
  {
    key: "customer",
    label: t("table.headers.customer"),
    labelClassName: "w-[20%]",
    render: (customer) => <div className="truncate">{customer.email}</div>,
  },
  {
    key: "facilityAmount",
    label: t("table.headers.facilityAmount"),
    labelClassName: "w-[15%]",
    render: (amount) => <Balance amount={amount} currency="usd" />,
    sortable: true,
  },
  {
    key: "collateralizationState",
    label: t("table.headers.collateralizationState"),
    labelClassName: "w-[20%]",
    render: (state) => <PendingFacilityCollateralizationStateLabel state={state} />,
    filterValues: Object.values(PendingCreditFacilityCollateralizationState),
    filterLabel: (state) => (
      <PendingFacilityCollateralizationStateLabel state={state} plain />
    ),
  },
  {
    key: "createdAt",
    label: t("table.headers.createdAt"),
    labelClassName: "w-[15%]",
    render: (date) => <DateWithTooltip value={date} />,
    sortable: true,
  },
]
