"use client"

import { useState } from "react"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import DateWithTooltip from "@lana/web/components/date-with-tooltip"

import { DisbursalStatusBadge } from "./status-badge"

import {
  CreditFacilityDisbursal,
  DisbursalStatus,
  DisbursalsFilter,
  useDisbursalsQuery,
} from "@/lib/graphql/generated"

import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"
import Balance from "@/components/balance/balance"

gql`
  query Disbursals($first: Int!, $after: String, $filter: DisbursalsFilter) {
    disbursals(first: $first, after: $after, filter: $filter) {
      edges {
        node {
          id
          creditFacilityDisbursalId
          publicId
          amount
          createdAt
          status
          creditFacility {
            customer {
              email
            }
          }
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
  const [filter, setFilter] = useState<DisbursalsFilter | null>(null)

  const { data, loading, error, fetchMore } = useDisbursalsQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
      filter,
    },
  })

  const columns: Column<CreditFacilityDisbursal>[] = [
    {
      key: "status",
      label: t("table.headers.status"),
      labelClassName: "w-[15%]",
      render: (status) => <DisbursalStatusBadge status={status} />,
      filterValues: Object.values(DisbursalStatus),
    },
    {
      key: "creditFacility",
      label: t("table.headers.customer"),
      labelClassName: "w-[35%]",
      render: (creditFacility) => (
        <div className="truncate">{creditFacility.customer.email}</div>
      ),
    },
    {
      key: "amount",
      label: t("table.headers.amount"),
      labelClassName: "w-[25%]",
      render: (amount) => <Balance amount={amount} currency="usd" />,
    },
    {
      key: "createdAt",
      label: t("table.headers.createdAt"),
      labelClassName: "w-[25%]",
      render: (date) => <DateWithTooltip value={date} />,
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
        navigateTo={(disbursal) => `/disbursals/${disbursal.publicId}`}
        onFilter={(filters) => {
          const f = filters as DisbursalsFilter
          setFilter(Object.keys(f).length > 0 ? f : null)
        }}
      />
    </div>
  )
}

export default Disbursals
