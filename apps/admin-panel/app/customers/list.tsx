"use client"

import { gql } from "@apollo/client"
import { useState } from "react"
import { useTranslations } from "next-intl"

import {
  CustomerStatus,
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

const CustomersList = () => {
  const t = useTranslations("Customers")

  const [sortBy, setSortBy] = useState<CustomersSort | null>(null)
  const [filter, setFilter] = useState<CustomersFilter | null>(null)

  const { data, loading, error, fetchMore } = useCustomersQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
      sort: sortBy,
      filter: filter,
    },
  })

  const columns: Column<Customer>[] = [
    { key: "email", label: t("columns.email"), sortable: true },
    { key: "telegramId", label: t("columns.telegramId"), sortable: true },
    {
      key: "status",
      label: t("columns.status"),
      filterValues: Object.values(CustomerStatus),
      render: (status) => (
        <div
          className={
            status === CustomerStatus.Inactive ? "text-error font-medium" : undefined
          }
        >
          {status === CustomerStatus.Active
            ? t("status.verified")
            : t("status.notVerified")}
        </div>
      ),
    },
    {
      key: "depositAccount",
      label: t("columns.depositAccount"),
      render: (depositAccount) =>
        depositAccount?.balance?.settled ? (
          <Balance amount={depositAccount?.balance?.settled} currency="usd" />
        ) : (
          <></>
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

export default CustomersList
