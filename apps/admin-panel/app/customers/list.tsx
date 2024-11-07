"use client"

import { gql } from "@apollo/client"
import { useRouter } from "next/navigation"

import { AccountStatus, Customer, useCustomersQuery } from "@/lib/graphql/generated"

import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/new/paginated-table"

gql`
  query Customers($first: Int!, $after: String, $sort: CustomersSort) {
    customers(first: $first, after: $after, sort: $sort) {
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

          subjectCanRecordDeposit
          subjectCanInitiateWithdrawal
          subjectCanCreateCreditFacility
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
  const router = useRouter()

  const { data, loading, error, fetchMore } = useCustomersQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
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
        onClick={(customer) => {
          router.push(`/customers/${customer.customerId}`)
        }}
      />
    </div>
  )
}

export default Customers

const columns: Column<Customer>[] = [
  { key: "email", label: "Email" },
  {
    key: "status",
    label: "KYC Status",
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
