"use client"

import { gql } from "@apollo/client"

import { Deposit, useDepositsQuery } from "@/lib/graphql/generated"

import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"

import Balance from "@/components/balance/balance"

gql`
  query Deposits($first: Int!, $after: String) {
    deposits(first: $first, after: $after) {
      pageInfo {
        hasPreviousPage
        hasNextPage
        startCursor
        endCursor
      }
      edges {
        cursor
        node {
          depositId
          amount
          reference
        }
      }
    }
  }
`

const Deposits = () => {
  const { data, loading, error, fetchMore } = useDepositsQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
    },
  })

  return (
    <div>
      {error && <p className="text-destructive text-sm">{error?.message}</p>}
      <PaginatedTable<Deposit>
        columns={columns}
        data={data?.deposits as PaginatedData<Deposit>}
        loading={loading}
        fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
        pageSize={DEFAULT_PAGESIZE}
      />
    </div>
  )
}

export default Deposits

const columns: Column<Deposit>[] = [
  { key: "customer", label: "Customer", render: (customer) => customer.email },
  {
    key: "reference",
    label: "Reference",
  },
  {
    key: "amount",
    label: "Amount",
    render: (amount) => <Balance amount={amount} currency="usd" />,
  },
]
