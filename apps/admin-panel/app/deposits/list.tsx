"use client"

import { useState } from "react"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import { DepositStatusBadge } from "./status-badge"

import {
  Deposit,
  DepositStatus,
  DepositsFilter,
  DepositsSort,
  SortDirection,
  useDepositsQuery,
} from "@/lib/graphql/generated"

import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"
import Balance from "@/components/balance/balance"
import { PublicIdBadge } from "@/components/public-id-badge"
import { camelToScreamingSnake } from "@/lib/utils"

gql`
  fragment DepositFields on Deposit {
    id
    depositId
    publicId
    reference
    createdAt
    amount
    status
    account {
      customer {
        customerId
        email
      }
    }
  }

  query Deposits($first: Int!, $after: String, $sort: DepositsSort, $filter: DepositsFilter) {
    deposits(first: $first, after: $after, sort: $sort, filter: $filter) {
      pageInfo {
        hasPreviousPage
        hasNextPage
        startCursor
        endCursor
      }
      edges {
        cursor
        node {
          ...DepositFields
        }
      }
    }
  }
`

const Deposits = () => {
  const t = useTranslations("Deposits.table")
  const [filter, setFilter] = useState<DepositsFilter | null>(null)
  const [sortBy, setSortBy] = useState<DepositsSort | null>(null)

  const { data, loading, error, fetchMore } = useDepositsQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
      sort: sortBy,
      filter,
    },
  })

  return (
    <div>
      {error && <p className="text-destructive text-sm">{error?.message}</p>}
      <PaginatedTable<Deposit>
        columns={columns(t)}
        data={data?.deposits as PaginatedData<Deposit>}
        loading={loading}
        fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
        pageSize={DEFAULT_PAGESIZE}
        navigateTo={(deposit) => `/deposits/${deposit.publicId}`}
        onSort={(column, direction) => {
          setSortBy({
            by: camelToScreamingSnake(column as string) as DepositsSort["by"],
            direction: direction as SortDirection,
          })
        }}
        onFilter={(filters) => {
          const f = filters as DepositsFilter
          setFilter(Object.keys(f).length > 0 ? f : null)
        }}
      />
    </div>
  )
}

export default Deposits

const columns = (t: ReturnType<typeof useTranslations>): Column<Deposit>[] => [
  {
    key: "publicId",
    label: t("headers.depositId"),
    sortable: true,
    render: (publicId) => <PublicIdBadge publicId={publicId} />,
  },
  {
    key: "account",
    label: t("headers.customer"),
    render: (account) => account.customer.email,
  },
  {
    key: "reference",
    label: t("headers.reference"),
    render: (reference, deposit) =>
      reference === deposit.depositId ? t("values.na") : reference,
  },
  {
    key: "amount",
    label: t("headers.amount"),
    sortable: true,
    render: (amount) => <Balance amount={amount} currency="usd" />,
  },
  {
    key: "status",
    label: t("headers.status"),
    render: (status) => <DepositStatusBadge status={status} />,
    filterValues: Object.values(DepositStatus),
  },
]
