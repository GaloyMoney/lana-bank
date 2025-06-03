"use client"

import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import DateWithTooltip from "@lana/web/components/date-with-tooltip"

import { DisbursalStatusBadge } from "./status-badge"

import { CreditFacilityDisbursal, useDisbursalsQuery } from "@/lib/graphql/generated"

import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"
import Balance from "@/components/balance/balance"

gql`
  query Disbursals($first: Int!, $after: String) {
    disbursals(first: $first, after: $after) {
      edges {
        node {
          id
          disbursalId
          amount
          createdAt
          status
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

const Disbursals = () => {
  const t = useTranslations("Disbursals")
  const { data, loading, error, fetchMore } = useDisbursalsQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
    },
  })

  const columns: Column<CreditFacilityDisbursal>[] = [
    {
      key: "amount",
      label: t("table.headers.amount"),
      render: (amount) => <Balance amount={amount} currency="usd" />,
    },
    {
      key: "createdAt",
      label: t("table.headers.createdAt"),
      render: (date) => <DateWithTooltip value={date} />,
    },
    {
      key: "status",
      label: t("table.headers.status"),
      render: (status) => <DisbursalStatusBadge status={status} />,
    },
  ]

  return (
    <div>
      {error && <p className="text-destructive text-sm">{t("errors.general")}</p>}
      <PaginatedTable<CreditFacilityDisbursal>
        columns={columns}
        data={data?.disbursals as PaginatedData<CreditFacilityDisbursal>}
        loading={loading}
        fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
        pageSize={DEFAULT_PAGESIZE}
        navigateTo={(disbursal) => `/disbursals/${disbursal.disbursalId}`}
      />
    </div>
  )
}

export default Disbursals
