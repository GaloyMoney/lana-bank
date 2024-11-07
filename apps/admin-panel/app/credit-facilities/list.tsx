/* eslint-disable @typescript-eslint/no-explicit-any */
"use client"

import { gql } from "@apollo/client"
import { useRouter } from "next/navigation"
import { useState } from "react"

import { LoanAndCreditFacilityStatusBadge } from "../loans/status-badge"

import {
  CreditFacilitiesSort,
  CreditFacility,
  SortDirection,
  CreditFacilityStatus,
  CollateralizationState,
  useCreditFacilitiesQuery,
  useCreditFacilitiesForStatusQuery,
  useCreditFacilitiesForCollateralizationStateQuery,
} from "@/lib/graphql/generated"

import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
} from "@/components/new/paginated-table"
import Balance from "@/components/balance/balance"
import {
  camelToScreamingSnake,
  formatCollateralizationState,
  formatDate,
} from "@/lib/utils"

gql`
  fragment CreditFacilitiesFields on CreditFacility {
    id
    creditFacilityId
    collateralizationState
    createdAt
    status
    facilityAmount
    collateral
    currentCvl {
      disbursed
      total
    }
    customer {
      customerId
      email
    }
    balance {
      outstanding {
        usdBalance
      }
    }
  }

  query CreditFacilities($first: Int!, $after: String, $sort: CreditFacilitiesSort) {
    creditFacilities(first: $first, after: $after, sort: $sort) {
      edges {
        cursor
        node {
          ...CreditFacilitiesFields
        }
      }
      pageInfo {
        endCursor
        hasNextPage
      }
    }
  }

  query CreditFacilitiesForStatus(
    $first: Int!
    $after: String
    $sort: CreditFacilitiesSort
    $status: CreditFacilityStatus!
  ) {
    creditFacilitiesForStatus(
      first: $first
      after: $after
      sort: $sort
      status: $status
    ) {
      edges {
        cursor
        node {
          ...CreditFacilitiesFields
        }
      }
      pageInfo {
        endCursor
        hasNextPage
      }
    }
  }

  query CreditFacilitiesForCollateralizationState(
    $first: Int!
    $after: String
    $sort: CreditFacilitiesSort
    $collateralizationState: CollateralizationState!
  ) {
    creditFacilitiesForCollateralizationState(
      first: $first
      after: $after
      sort: $sort
      collateralizationState: $collateralizationState
    ) {
      edges {
        cursor
        node {
          ...CreditFacilitiesFields
        }
      }
      pageInfo {
        endCursor
        hasNextPage
      }
    }
  }
`

const CreditFacilities = () => {
  const router = useRouter()
  const [sortBy, setSortBy] = useState<CreditFacilitiesSort>()
  const [filters, setFilters] = useState<{
    status?: CreditFacilityStatus
    collateralizationState?: CollateralizationState
  }>({})

  // Main query without filters
  const {
    data: dataAll,
    loading: loadingAll,
    error: errorAll,
    fetchMore: fetchMoreAll,
  } = useCreditFacilitiesQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
      sort: sortBy,
    },
    skip: filters.status != null || filters.collateralizationState != null,
  })

  // Query filtered by status
  const {
    data: dataStatus,
    loading: loadingStatus,
    error: errorStatus,
    fetchMore: fetchMoreStatus,
  } = useCreditFacilitiesForStatusQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
      sort: sortBy,
      status: filters.status!,
    },
    skip: filters.status == null || filters.collateralizationState != null,
  })

  // Query filtered by collateralization state
  const {
    data: dataCollateralization,
    loading: loadingCollateralization,
    error: errorCollateralization,
    fetchMore: fetchMoreCollateralization,
  } = useCreditFacilitiesForCollateralizationStateQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
      sort: sortBy,
      collateralizationState: filters.collateralizationState!,
    },
    skip: filters.collateralizationState == null || filters.status != null,
  })

  // Decide which data to use based on filters
  let data: any
  let loading = false
  let error: any
  let fetchMore: any

  if (filters.status) {
    data = dataStatus?.creditFacilitiesForStatus
    loading = loadingStatus
    error = errorStatus
    fetchMore = (cursor: string) => fetchMoreStatus({ variables: { after: cursor } })
  } else if (filters.collateralizationState) {
    data = dataCollateralization?.creditFacilitiesForCollateralizationState
    loading = loadingCollateralization
    error = errorCollateralization
    fetchMore = (cursor: string) =>
      fetchMoreCollateralization({ variables: { after: cursor } })
  } else {
    data = dataAll?.creditFacilities
    loading = loadingAll
    error = errorAll
    fetchMore = (cursor: string) => fetchMoreAll({ variables: { after: cursor } })
  }

  return (
    <div>
      {error && <p className="text-destructive text-sm">{error?.message}</p>}
      <PaginatedTable<CreditFacility>
        columns={columns}
        data={data}
        loading={loading}
        fetchMore={async (cursor) => fetchMore(cursor)}
        pageSize={DEFAULT_PAGESIZE}
        onClick={(facility) => {
          router.push(`/credit-facilities/${facility.creditFacilityId}`)
        }}
        onSort={(column, direction) => {
          setSortBy({
            by: (column === "currentCvl"
              ? "CVL"
              : camelToScreamingSnake(column)) as CreditFacilitiesSort["by"],
            direction: direction as SortDirection,
          })
        }}
        onFilter={(column, value) => {
          // Reset all filters except the one being applied
          setFilters({ [column]: value })
        }}
      />
    </div>
  )
}

export default CreditFacilities

const columns: Column<CreditFacility>[] = [
  { key: "customer", label: "Customer", render: (customer) => customer.email },
  {
    key: "status",
    label: "Status",
    render: (status) => <LoanAndCreditFacilityStatusBadge status={status} />,
    filterValues: Object.values(CreditFacilityStatus),
  },
  {
    key: "balance",
    label: "Outstanding",
    render: (balance) => (
      <Balance amount={balance.outstanding.usdBalance} currency="usd" />
    ),
  },
  {
    key: "collateralizationState",
    label: "Collateralization State",
    render: (state) => formatCollateralizationState(state),
    filterValues: Object.values(CollateralizationState),
  },
  {
    key: "currentCvl",
    label: "CVL",
    render: (cvl) => `${cvl.disbursed}%`,
    sortable: true,
  },
  {
    key: "createdAt",
    label: "Created At",
    render: (date) => formatDate(date),
    sortable: true,
  },
]
