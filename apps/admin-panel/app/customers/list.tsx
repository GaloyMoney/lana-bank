"use client"

import { gql } from "@apollo/client"
import { useRouter } from "next/navigation"

import { AccountStatus, Customer, useCustomersQuery } from "@/lib/graphql/generated"

import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/new/paginated-table"

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/primitive/card"

gql`
  query Customers($first: Int!, $after: String) {
    customersByEmail(first: $first, after: $after) {
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

  const { data, fetchMore } = useCustomersQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
    },
  })

  return (
    <Card>
      <CardHeader>
        <CardTitle>Customers</CardTitle>
        <CardDescription>
          Individuals or entities who hold accounts, loans, or credit facilities with the
          bank
        </CardDescription>
      </CardHeader>
      <CardContent>
        {data && (
          <PaginatedTable<Customer>
            columns={columns}
            data={data?.customersByEmail as PaginatedData<Customer>}
            fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
            pageSize={DEFAULT_PAGESIZE}
            onClick={(customer) => {
              router.push(`/customers/${customer.customerId}`)
            }}
          />
        )}
      </CardContent>
    </Card>
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
    sortable: true,
  },
]
