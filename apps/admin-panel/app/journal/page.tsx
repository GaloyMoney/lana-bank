"use client"

import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@lana/web/ui/card"

import { JournalEntry, useJournalEntriesQuery } from "@/lib/graphql/generated"

import { formatDate, formatDirection } from "@/lib/utils"

import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"
import Balance from "@/components/balance/balance"

gql`
  query JournalEntries($first: Int!, $after: String) {
    journalEntries(first: $first, after: $after) {
      edges {
        cursor
        node {
          id
          entryId
          entryType
          description
          direction
          createdAt
          amount {
            ... on UsdAmount {
              usd
            }
            ... on BtcAmount {
              btc
            }
          }
        }
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

const JournalPage: React.FC = () => {
  const t = useTranslations("Journal")

  const { data, loading, error, fetchMore } = useJournalEntriesQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
    },
  })

  const columns: Column<JournalEntry>[] = [
    {
      key: "createdAt",
      label: t("table.createdAt"),
      render: (date: string) => formatDate(date),
    },
    {
      key: "entryType",
      label: t("table.entryType"),
    },
    {
      key: "direction",
      label: t("table.direction"),
      render: (direction: string) => formatDirection(direction),
    },
    {
      key: "amount",
      label: t("table.amount"),
      render: (amount) => {
        if (amount.__typename === "UsdAmount") {
          return <Balance currency="usd" amount={amount.usd} />
        } else if (amount.__typename === "BtcAmount") {
          return <Balance currency="btc" amount={amount.btc} />
        }
      },
    },
    {
      key: "description",
      label: t("table.description"),
      render: (description?: string | null) => {
        if (description) return description
        return "-"
      },
    },
  ]

  return (
    <Card className="mt-2">
      <CardHeader>
        <CardTitle>{t("title")}</CardTitle>
        <CardDescription>{t("description")}</CardDescription>
      </CardHeader>
      <CardContent>
        {error ? (
          <p className="text-destructive text-sm">{error?.message}</p>
        ) : (
          <PaginatedTable<JournalEntry>
            columns={columns}
            data={data?.journalEntries as PaginatedData<JournalEntry>}
            pageSize={DEFAULT_PAGESIZE}
            fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
            loading={loading}
            noDataText={t("noTableData")}
          />
        )}
      </CardContent>
    </Card>
  )
}

export default JournalPage
