"use client"

import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import { DepositStatusBadge } from "./status-badge"

import { Deposit, useDepositsQuery } from "@/lib/graphql/generated"

import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"
import Balance from "@/components/balance/balance"
import { PublicIdBadge } from "@/components/public-id-badge"

gql`
  fragment DepositFields on Deposit {
    id
    depositId
    publicId
    reference
    createdAt
    amount
    status
  }

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
          ...DepositFields
        }
      }
    }
  }
`

const Deposits = () => {
  const t = useTranslations("Deposits.table")
  const { data, loading, error, fetchMore } = useDepositsQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
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
      />
    </div>
  )
}

export default Deposits

const columns = (t: ReturnType<typeof useTranslations>): Column<Deposit>[] => [
  {
    key: "publicId",
    label: t("headers.depositId"),
    render: (publicId) => <PublicIdBadge publicId={publicId} />,
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
    render: (amount) => <Balance amount={amount} currency="usd" />,
  },
  {
    key: "status",
    label: t("headers.status"),
    render: (status) => <DepositStatusBadge status={status} />,
  },
]
