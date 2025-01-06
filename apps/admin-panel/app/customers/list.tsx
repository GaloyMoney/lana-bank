"use client"

import { gql } from "@apollo/client"
import { useState } from "react"

import {
  AccountStatus,
  Customer,
  CustomersFilter,
  CustomersSort,
  SortDirection,
  useCustomersQuery,
} from "@/lib/graphql/generated"

import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"
import { camelToScreamingSnake } from "@/lib/utils"
import Balance from "@/components/balance/balance"

gql`
  query Customers(
    $first: Int!
    $after: String
    $sort: CustomersSort
    $filter: CustomersFilter
  ) {
    customers(first: $first, after: $after, sort: $sort, filter: $filter) {
      edges {
        node {
          id
          customerId
          status
          level
          email
          telegramId
          applicantId
          depositAccount {
            balance {
              settled
              pending
            }
          }
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

const Customers = () => {
  const [sortBy, setSortBy] = useState<CustomersSort | null>(null)
  const [filter, setFilter] = useState<CustomersFilter | null>(null)

  const { data, loading, error, fetchMore } = useCustomersQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
      sort: sortBy,
      filter: filter,
    },
  })

  return (
    <div>
      {error && <p className="text-destructive text-sm">{error?.message}</p>}
      <PaginatedTable<Customer>
        columns={columns}
        data={data?.customers as PaginatedData<Customer>}
        loading={loading}
        fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
        pageSize={DEFAULT_PAGESIZE}
        navigateTo={(customer) => `/customers/${customer.customerId}`}
        onSort={(column, direction) => {
          setSortBy({
            by: camelToScreamingSnake(column) as CustomersSort["by"],
            direction: direction as SortDirection,
          })
        }}
        onFilter={(column, value) => {
          if (value)
            setFilter({
              field: (column === "status"
                ? "ACCOUNT_STATUS"
                : null) as CustomersFilter["field"],
              [column]: value,
            })
          else setFilter(null)
        }}
      />
    </div>
  )
}

const columns: Column<Customer>[] = [
  { key: "email", label: "Email", sortable: true },
  { key: "telegramId", label: "Telegram", sortable: true },
  {
    key: "status",
    label: "KYC Status",
    filterValues: Object.values(AccountStatus),
    render: (status) => (
      <div
        className={
          status === AccountStatus.Inactive ? "text-error font-medium" : undefined
        }
      >
        {status === AccountStatus.Active ? "Verified" : "Not Verified"}
      </div>
    ),
  },
  {
    key: "depositAccount",
    label: "USD Balance",
    render: (depositAccount) => (
      <Balance amount={depositAccount?.balance?.settled ?? 0} currency="usd" />
    ),
  },
]
export default Customers
