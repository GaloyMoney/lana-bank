"use client"

import { useState } from "react"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import { ActivityStatusBadge } from "../customers/activity-status-badge"

import { DepositAccountStatusBadge } from "./status-badge"

import {
  Activity,
  DepositAccount,
  DepositAccountStatus,
  DepositAccountsFilter,
  DepositAccountsSort,
  SortDirection,
  useDepositAccountsQuery,
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
  fragment DepositAccountFields on DepositAccount {
    id
    publicId
    createdAt
    status
    activity
    balance {
      settled
      pending
    }
    customer {
      customerId
      email
      publicId
    }
  }

  query DepositAccounts($first: Int!, $after: String, $sort: DepositAccountsSort, $filter: DepositAccountsFilter) {
    depositAccounts(first: $first, after: $after, sort: $sort, filter: $filter) {
      pageInfo {
        hasPreviousPage
        hasNextPage
        startCursor
        endCursor
      }
      edges {
        cursor
        node {
          ...DepositAccountFields
        }
      }
    }
  }
`

const DepositAccounts = () => {
  const t = useTranslations("DepositAccounts.table")
  const tActivity = useTranslations("Customers.status")
  const tStatus = useTranslations("DepositAccounts.status")
  const [filter, setFilter] = useState<DepositAccountsFilter | null>(null)
  const [sortBy, setSortBy] = useState<DepositAccountsSort | null>(null)

  const { data, loading, error, fetchMore } = useDepositAccountsQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
      sort: sortBy,
      filter,
    },
  })

  return (
    <div>
      {error && <p className="text-destructive text-sm">{error?.message}</p>}
      <PaginatedTable<DepositAccount>
        columns={columns(t, tActivity, tStatus)}
        data={data?.depositAccounts as PaginatedData<DepositAccount>}
        loading={loading}
        fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
        pageSize={DEFAULT_PAGESIZE}
        navigateTo={(depositAccount) => `/deposit-accounts/${depositAccount.publicId}`}
        onSort={(column, direction) => {
          setSortBy({
            by: camelToScreamingSnake(column as string) as DepositAccountsSort["by"],
            direction: direction as SortDirection,
          })
        }}
        onFilter={(filters) => {
          const f = filters as DepositAccountsFilter
          setFilter(Object.keys(f).length > 0 ? f : null)
        }}
      />
    </div>
  )
}

export default DepositAccounts

const columns = (
  t: ReturnType<typeof useTranslations>,
  tActivity: ReturnType<typeof useTranslations>,
  tStatus: ReturnType<typeof useTranslations>,
): Column<DepositAccount>[] => [
  {
    key: "publicId",
    label: t("headers.depositAccountId"),
    labelClassName: "w-[12%]",
    sortable: true,
    render: (publicId) => <PublicIdBadge publicId={publicId} />,
  },
  {
    key: "customer",
    label: t("headers.customer"),
    render: (customer) => <div className="truncate">{customer.email}</div>,
  },
  {
    key: "balance",
    label: t("headers.settledBalance"),
    render: (balance) => <Balance amount={balance.settled} currency="usd" />,
  },
  {
    key: "balance",
    label: t("headers.pendingBalance"),
    render: (balance) => <Balance amount={balance.pending} currency="usd" />,
  },
  {
    key: "status",
    label: t("headers.status"),
    render: (status) => <DepositAccountStatusBadge status={status} />,
    filterValues: Object.values(DepositAccountStatus),
    filterLabel: (status) => tStatus(status.toLowerCase()),
  },
  {
    key: "activity",
    label: t("headers.activity"),
    render: (activity) => <ActivityStatusBadge status={activity} />,
    filterValues: Object.values(Activity),
    filterLabel: (activity) => tActivity(activity.toLowerCase()),
  },
]
