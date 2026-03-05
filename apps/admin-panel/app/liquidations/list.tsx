"use client"

import { useState } from "react"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import DateWithTooltip from "@lana/web/components/date-with-tooltip"

import { LiquidationStatusBadge } from "./status-badge"

import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"
import Balance from "@/components/balance/balance"
import { PublicIdBadge } from "@/components/public-id-badge"
import {
  Liquidation,
  LiquidationsSort,
  SortDirection,
  useLiquidationsQuery,
} from "@/lib/graphql/generated"
import { camelToScreamingSnake } from "@/lib/utils"

gql`
  fragment LiquidationListFields on Liquidation {
    id
    liquidationId
    expectedToReceive
    sentTotal
    amountReceived
    createdAt
    completed
    collateral {
      creditFacility {
        publicId
      }
    }
  }

  query Liquidations($first: Int!, $after: String, $sort: LiquidationsSort) {
    liquidations(first: $first, after: $after, sort: $sort) {
      edges {
        node {
          ...LiquidationListFields
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

const LiquidationsList = () => {
  const t = useTranslations("Liquidations")
  const [sortBy, setSortBy] = useState<LiquidationsSort | null>(null)

  const { data, loading, error, fetchMore } = useLiquidationsQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
      sort: sortBy,
    },
  })

  const columns: Column<Liquidation>[] = [
    {
      key: "completed",
      label: t("table.headers.status"),
      render: (completed) => <LiquidationStatusBadge completed={completed} />,
    },
    {
      key: "collateral",
      label: t("table.headers.creditFacility"),
      render: (collateral) => (
        <PublicIdBadge publicId={String(collateral.creditFacility?.publicId)} />
      ),
    },
    {
      key: "expectedToReceive",
      label: t("table.headers.expectedToReceive"),
      render: (amount) => <Balance amount={amount} currency="usd" />,
      sortable: true,
    },
    {
      key: "sentTotal",
      label: t("table.headers.sentTotal"),
      render: (amount) => <Balance amount={amount} currency="btc" />,
      sortable: true,
    },
    {
      key: "amountReceived",
      label: t("table.headers.amountReceived"),
      render: (amount) => <Balance amount={amount} currency="usd" />,
      sortable: true,
    },
    {
      key: "createdAt",
      label: t("table.headers.createdAt"),
      render: (date) => <DateWithTooltip value={date} />,
      sortable: true,
    },
  ]

  return (
    <div>
      {error && <p className="text-destructive text-sm">{t("errors.general")}</p>}
      <PaginatedTable<Liquidation>
        columns={columns}
        data={data?.liquidations as PaginatedData<Liquidation>}
        loading={loading}
        fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
        pageSize={DEFAULT_PAGESIZE}
        navigateTo={(liquidation) => `/liquidations/${liquidation.liquidationId}`}
        onSort={(column, direction) => {
          setSortBy({
            by: camelToScreamingSnake(column) as LiquidationsSort["by"],
            direction: direction as SortDirection,
          })
        }}
      />
    </div>
  )
}

export default LiquidationsList
