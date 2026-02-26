"use client"

import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import { WithdrawalStatusBadge } from "./status-badge"

import { Withdrawal, useWithdrawalsQuery } from "@/lib/graphql/generated"

import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"
import Balance from "@/components/balance/balance"
import { PublicIdBadge } from "@/components/public-id-badge"

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
      customerId
    }
  }

  query Withdrawals($first: Int!, $after: String) {
    withdrawals(first: $first, after: $after) {
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
  const { data, loading, error, fetchMore } = useWithdrawalsQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
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
    render: (account) => account.customerId,
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
    render: (amount) => <Balance amount={amount} currency="usd" />,
  },
  {
    key: "status",
    label: t("headers.status"),
    render: (status) => <WithdrawalStatusBadge status={status} />,
  },
]
