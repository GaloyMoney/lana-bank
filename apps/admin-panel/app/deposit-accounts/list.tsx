"use client"

import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import { DepositAccountStatusBadge } from "./status-badge"

import { DepositAccount, useDepositAccountsQuery } from "@/lib/graphql/generated"

import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"
import Balance from "@/components/balance/balance"
import { PublicIdBadge } from "@/components/public-id-badge"

gql`
  fragment DepositAccountFields on DepositAccount {
    id
    publicId
    createdAt
    status
    balance {
      settled
      pending
    }
  }

  query DepositAccounts($first: Int!, $after: String) {
    depositAccounts(first: $first, after: $after) {
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
  const { data, loading, error, fetchMore } = useDepositAccountsQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
    },
  })

  return (
    <div>
      {error && <p className="text-destructive text-sm">{error?.message}</p>}
      <PaginatedTable<DepositAccount>
        columns={columns(t)}
        data={data?.depositAccounts as PaginatedData<DepositAccount>}
        loading={loading}
        fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
        pageSize={DEFAULT_PAGESIZE}
        navigateTo={(depositAccount) => `/deposit-accounts/${depositAccount.publicId}`}
      />
    </div>
  )
}

export default DepositAccounts

const columns = (t: ReturnType<typeof useTranslations>): Column<DepositAccount>[] => [
  {
    key: "publicId",
    label: t("headers.depositAccountId"),
    render: (publicId) => <PublicIdBadge publicId={publicId} />,
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
  },
]
