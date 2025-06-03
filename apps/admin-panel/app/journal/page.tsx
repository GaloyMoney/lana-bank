"use client"

import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"
import Link from "next/link"

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@lana/web/ui/card"

import DateWithTooltip from "@lana/web/components/date-with-tooltip"

import {
  DebitOrCredit,
  JournalEntry,
  LedgerTransaction,
  useJournalEntriesQuery,
} from "@/lib/graphql/generated"

import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"
import Balance from "@/components/balance/balance"
import { formatDate } from "@/lib/utils"

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
          ledgerAccount {
            id
            code
            name
            closestAccountWithCode {
              code
            }
          }
          ledgerTransaction {
            id
            ledgerTransactionId
            description
            effective
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
      render: (date: string) => <DateWithTooltip value={date} />,
    },
    {
      key: "ledgerTransaction",
      label: t("table.effective"),
      render: (transaction: LedgerTransaction) =>
        formatDate(transaction.effective, {
          includeTime: false,
        }),
    },
    {
      key: "ledgerTransaction",
      label: t("table.transaction"),
      render: (transaction) => {
        const transactionName = transaction.description || transaction.id
        return (
          <Link
            href={`/ledger-transaction/${transaction.ledgerTransactionId}`}
            className="hover:underline"
          >
            {transactionName}
          </Link>
        )
      },
    },
    {
      key: "entryType",
      label: t("table.entryType"),
    },
    {
      key: "ledgerAccount",
      label: t("table.name"),
      render: (account) => {
        const accountName = account.name || account.code
        return (
          <Link
            href={`/ledger-account/${account.code || account.id}`}
            className="hover:underline"
          >
            {accountName}
          </Link>
        )
      },
    },
    {
      key: "direction",
      label: t("table.direction"),
      render: (direction: DebitOrCredit) =>
        direction === DebitOrCredit.Debit
          ? t("debitOrCredit.debit")
          : t("debitOrCredit.credit"),
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
    {
      key: "ledgerAccount",
      label: t("table.closestAccountWithCode"),
      render: (account) => {
        const closestAccountWithCode = account.closestAccountWithCode?.code
        return (
          <Link
            href={`/ledger-account/${closestAccountWithCode}`}
            className="hover:underline"
          >
            {closestAccountWithCode}
          </Link>
        )
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
