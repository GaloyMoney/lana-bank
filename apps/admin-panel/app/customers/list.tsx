"use client"

import { gql } from "@apollo/client"
import { useState } from "react"
import { useTranslations } from "next-intl"

import { CustomerStatusBadge } from "./customer-status-badge"

import {
  CustomerStatus,
  CustomerType,
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
          publicId
          status
          activity
          level
          email
          telegramHandle
          applicantId
          customerType
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

const CustomersList = () => {
  const t = useTranslations("Customers")

  const [sortBy, setSortBy] = useState<CustomersSort | null>(null)
  const [filter, setFilter] = useState<CustomersFilter | null>(null)

  const { data, loading, error, fetchMore } = useCustomersQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
      sort: sortBy,
      filter,
    },
  })

  const columns: Column<Customer>[] = [
    {
      key: "email",
      label: t("columns.email"),
      labelClassName: "w-[30%]",
    },
    {
      key: "telegramHandle",
      label: t("columns.telegramHandle"),
      labelClassName: "w-[30%]",
    },
    {
      key: "status",
      label: t("columns.status"),
      filterValues: Object.values(CustomerStatus),
      render: (status) => <CustomerStatusBadge status={status} />,
    },
    {
      key: "customerType",
      label: t("columns.customerType"),
      filterValues: Object.values(CustomerType),
    },
    {
      key: "depositAccount",
      label: t("columns.depositAccount"),
      render: (depositAccount) =>
        depositAccount?.balance?.settled ? (
          <Balance amount={depositAccount?.balance?.settled} currency="usd" />
        ) : (
          <>-</>
        ),
    },
  ]

  return (
    <div>
      {error && <p className="text-destructive text-sm">{error?.message}</p>}
      <PaginatedTable<Customer>
        columns={columns}
        data={data?.customers as PaginatedData<Customer>}
        loading={loading}
        fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
        pageSize={DEFAULT_PAGESIZE}
        navigateTo={(customer) => `/customers/${customer.publicId}`}
        onSort={(column, direction) => {
          setSortBy({
            by: camelToScreamingSnake(column) as CustomersSort["by"],
            direction: direction as SortDirection,
          })
        }}
        onFilter={(filters) => {
          const f = filters as CustomersFilter
          setFilter(Object.keys(f).length > 0 ? f : null)
        }}
      />
    </div>
  )
}

export default CustomersList
