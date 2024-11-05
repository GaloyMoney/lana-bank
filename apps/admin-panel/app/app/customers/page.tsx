"use client"

import { gql } from "@apollo/client"

import PaginatedTable, { Column, PaginatedData } from "@/components/paginated-table"
import { AccountStatus, Customer, useCustomersQuery } from "@/lib/graphql/generated"

gql`
  query Customers($first: Int!, $after: String) {
    customers(first: $first, after: $after) {
      edges {
        node {
          id
          customerId
          status
          level
          email
          telegramId
          applicantId
          balance {
            checking {
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
  const { data, fetchMore } = useCustomersQuery({
    variables: {
      first: 2,
    },
  })

  return (
    <div className="bg-page rounded-md p-[10px] flex flex-col gap-1 border">
      <div className="text-title-md">Customers</div>
      <div className="!text-body text-body-sm">
        Individuals or entities who hold accounts, loans, or credit facilities with the
        bank
      </div>
      {data && (
        <PaginatedTable<Customer>
          columns={columns}
          data={data?.customers as PaginatedData<Customer>}
          fetchMore={(cursor) => fetchMore({ variables: { after: cursor } })}
          pageSize={2}
        />
      )}
    </div>
  )
}

export default Customers

const columns: Column<Customer>[] = [
  { key: "email", label: "Email" },
  {
    key: "status",
    label: "KYC Status",
    filterValues: [AccountStatus.Active, AccountStatus.Inactive],
    render: (status) => (
      <div
        className={
          (status === AccountStatus.Inactive && "text-error font-medium") || undefined
        }
      >
        {status === AccountStatus.Active ? "Verified" : "Not Verified"}
      </div>
    ),
  },
  {
    key: "balance",
    label: "USD Balance",
    render: (balance) => <div>${balance.checking.settled}</div>,
  },
]
