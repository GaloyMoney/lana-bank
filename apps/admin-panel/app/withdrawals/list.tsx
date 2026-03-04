"use client"

import { useState } from "react"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import { WithdrawalStatusBadge } from "./status-badge"

import {
  Withdrawal,
  WithdrawalStatus,
  WithdrawalsFilter,
  WithdrawalsSort,
  SortDirection,
  useWithdrawalsQuery,
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
  fragment WithdrawalFields on Withdrawal {
    id
    status
    reference
    withdrawalId
    publicId
    createdAt
    amount
    account {
      customer {
        customerId
        email
      }
    }
  }

  query Withdrawals($first: Int!, $after: String, $sort: WithdrawalsSort, $filter: WithdrawalsFilter) {
    withdrawals(first: $first, after: $after, sort: $sort, filter: $filter) {
      pageInfo {
        hasPreviousPage
        hasNextPage
        startCursor
        endCursor
      }
      edges {
        cursor
        node {
          ...WithdrawalFields
        }
      }
    }
  }
`

const Withdrawals = () => {
  const t = useTranslations("Withdrawals.table")
  const [filter, setFilter] = useState<WithdrawalsFilter | null>(null)
  const [sortBy, setSortBy] = useState<WithdrawalsSort | null>(null)

  const { data, loading, error, fetchMore } = useWithdrawalsQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
      sort: sortBy,
      filter,
    },
  })

  return (
    <div>
      {error && <p className="text-destructive text-sm">{error?.message}</p>}
      <PaginatedTable<Withdrawal>
        columns={columns(t)}
        data={data?.withdrawals as PaginatedData<Withdrawal>}
        loading={loading}
        fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
        pageSize={DEFAULT_PAGESIZE}
        navigateTo={(withdrawal) => `/withdrawals/${withdrawal.publicId}`}
        onSort={(column, direction) => {
          setSortBy({
            by: camelToScreamingSnake(column as string) as WithdrawalsSort["by"],
            direction: direction as SortDirection,
          })
        }}
        onFilter={(filters) => {
          const f = filters as WithdrawalsFilter
          setFilter(Object.keys(f).length > 0 ? f : null)
        }}
      />
    </div>
  )
}

export default Withdrawals

const columns = (t: ReturnType<typeof useTranslations>): Column<Withdrawal>[] => [
  {
    key: "publicId",
    label: t("headers.withdrawalId"),
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
    render: (reference, withdrawal) =>
      reference === withdrawal.withdrawalId ? t("values.na") : reference,
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
    render: (status) => <WithdrawalStatusBadge status={status} />,
    filterValues: Object.values(WithdrawalStatus),
  },
]
