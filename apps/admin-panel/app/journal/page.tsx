"use client"

import React, { useState, useMemo } from "react"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"
import Link from "next/link"
import { useRouter } from "next/navigation"
import { HiChevronLeft, HiChevronRight } from "react-icons/hi"

import { Card, CardDescription, CardHeader, CardTitle } from "@lana/web/ui/card"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@lana/web/ui/table"
import { Button } from "@lana/web/ui/button"
import { Separator } from "@lana/web/ui/separator"
import DateWithTooltip from "@lana/web/components/date-with-tooltip"
import { formatDate } from "@lana/web/utils"

import {
  DebitOrCredit,
  JournalEntriesQuery,
  useJournalEntriesQuery,
} from "@/lib/graphql/generated"
import Balance from "@/components/balance/balance"
import LayerLabel from "@/app/journal/layer-label"
import { TableLoadingSkeleton } from "@/components/table-loading-skeleton"

type JournalEntry = JournalEntriesQuery["journalEntries"]["edges"][number]["node"]

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
          layer
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
            ledgerAccountId
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

const PAGE_SIZE = 50

const getColumns = (t: ReturnType<typeof useTranslations>) => [
  {
    key: "createdAt",
    label: t("table.createdAt"),
    width: "w-[9%]",
    render: (entry: JournalEntry) => <DateWithTooltip value={entry.createdAt} />,
  },
  {
    key: "effective",
    label: t("table.effective"),
    width: "w-[9%]",
    render: (entry: JournalEntry) =>
      formatDate(entry.ledgerTransaction.effective, { includeTime: false }),
  },
  {
    key: "transaction",
    label: t("table.transaction"),
    width: "w-[15%]",
    render: (entry: JournalEntry) => (
      <Link
        href={`/ledger-transaction/${entry.ledgerTransaction.ledgerTransactionId}`}
        className="hover:underline"
      >
        {entry.ledgerTransaction.description || entry.ledgerTransaction.id}
      </Link>
    ),
  },
  {
    key: "entryType",
    label: t("table.entryType"),
    width: "w-[15%]",
    render: (entry: JournalEntry) => <span className="text-sm">{entry.entryType}</span>,
  },
  {
    key: "name",
    label: t("table.name"),
    width: "w-[15%]",
    render: (entry: JournalEntry) => (
      <Link
        href={`/ledger-accounts/${entry.ledgerAccount.code || entry.ledgerAccount.ledgerAccountId}`}
        className="hover:underline"
      >
        {entry.ledgerAccount.name || entry.ledgerAccount.code}
      </Link>
    ),
  },
  {
    key: "closestAccountWithCode",
    label: t("table.closestAccountWithCode"),
    width: "w-[10%]",
    render: (entry: JournalEntry) => (
      <Link
        href={`/ledger-accounts/${entry.ledgerAccount.closestAccountWithCode?.code}`}
        className="hover:underline"
      >
        {entry.ledgerAccount.closestAccountWithCode?.code}
      </Link>
    ),
  },
  {
    key: "layer",
    label: t("table.layer"),
    width: "w-[7%]",
    render: (entry: JournalEntry) => <LayerLabel value={entry.layer} />,
  },
  {
    key: "debit",
    label: t("table.debit"),
    width: "w-[10%]",
    render: (entry: JournalEntry) => {
      if (entry.direction !== DebitOrCredit.Debit) return null
      return entry.amount.__typename === "UsdAmount" ? (
        <Balance amount={entry.amount.usd} currency="usd" />
      ) : entry.amount.__typename === "BtcAmount" ? (
        <Balance amount={entry.amount.btc} currency="btc" />
      ) : null
    },
  },
  {
    key: "credit",
    label: t("table.credit"),
    width: "w-[10%]",
    render: (entry: JournalEntry) => {
      if (entry.direction !== DebitOrCredit.Credit) return null
      return entry.amount.__typename === "UsdAmount" ? (
        <Balance amount={entry.amount.usd} currency="usd" />
      ) : entry.amount.__typename === "BtcAmount" ? (
        <Balance amount={entry.amount.btc} currency="btc" />
      ) : null
    },
  },
]

const JournalPageCard: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const t = useTranslations("Journal")
  return (
    <Card className="mt-2 shadow-none">
      <CardHeader>
        <CardTitle>{t("title")}</CardTitle>
        <CardDescription>{t("description")}</CardDescription>
      </CardHeader>
      {children}
    </Card>
  )
}

const JournalPage: React.FC = () => {
  const t = useTranslations("Journal")
  const router = useRouter()
  const columns = getColumns(t)

  const { data, loading, error, fetchMore } = useJournalEntriesQuery({
    variables: { first: PAGE_SIZE },
  })

  const [currentPage, setCurrentPage] = useState(1)
  const [hoveredTxId, setHoveredTxId] = useState<string | null>(null)

  const displayData = useMemo(() => {
    if (!data?.journalEntries?.edges) return []
    const startIdx = (currentPage - 1) * PAGE_SIZE
    const endIdx = startIdx + PAGE_SIZE
    return data.journalEntries.edges.slice(startIdx, endIdx).map((edge) => edge.node)
  }, [data, currentPage])

  const handleNextPage = async () => {
    const totalDataLoaded = data?.journalEntries?.edges.length || 0
    const maxDataRequired = currentPage * PAGE_SIZE + PAGE_SIZE

    if (totalDataLoaded < maxDataRequired && data?.journalEntries?.pageInfo.hasNextPage) {
      await fetchMore({ variables: { after: data.journalEntries.pageInfo.endCursor } })
    }
    setCurrentPage(currentPage + 1)
  }

  const handlePreviousPage = () => {
    if (currentPage > 1) {
      setCurrentPage(currentPage - 1)
    }
  }

  if (loading) {
    return (
      <JournalPageCard>
        <TableLoadingSkeleton rows={PAGE_SIZE} columns={columns.length} />
      </JournalPageCard>
    )
  }

  if (error) {
    return (
      <JournalPageCard>
        <div className="p-6">
          <p className="text-destructive text-sm">{error.message}</p>
        </div>
      </JournalPageCard>
    )
  }

  if (!data?.journalEntries?.edges || data.journalEntries.edges.length === 0) {
    return (
      <JournalPageCard>
        <div className="p-6">
          <div className="text-sm">{t("noTableData")}</div>
        </div>
      </JournalPageCard>
    )
  }

  return (
    <JournalPageCard>
      <div className="overflow-x-auto rounded-md border">
        <Table className="table-fixed w-full">
          <TableHeader className="bg-secondary [&_tr:hover]:!bg-secondary">
            <TableRow>
              {columns.map((col) => (
                <TableHead key={col.key} className={col.width}>
                  {col.label}
                </TableHead>
              ))}
            </TableRow>
          </TableHeader>
          <TableBody>
            {displayData.map((entry, index) => {
              const isFirst = isFirstInGroup(displayData, index)
              const txId = entry.ledgerTransaction.ledgerTransactionId

              return (
                <React.Fragment key={entry.entryId}>
                  {isFirst && index > 0 && (
                    <TableRow className="h-3">
                      <TableCell
                        colSpan={columns.length}
                        className="p-0 border-t-2 border-b-2"
                      />
                    </TableRow>
                  )}
                  {isFirst && (
                    <TableRow
                      onClick={() => router.push(`/ledger-transaction/${txId}`)}
                      className="bg-muted/50 cursor-pointer"
                      onMouseEnter={() => setHoveredTxId(txId)}
                      onMouseLeave={() => setHoveredTxId(null)}
                    >
                      <TableCell
                        colSpan={columns.length}
                        className="p-2 font-semibold text-xs"
                      >
                        TXID: {txId}
                      </TableCell>
                    </TableRow>
                  )}
                  <TableRow className={hoveredTxId === txId ? "bg-muted/50" : ""}>
                    {columns.map((col) => (
                      <TableCell key={col.key}>
                        <div className="line-clamp-2">{col.render(entry)}</div>
                      </TableCell>
                    ))}
                  </TableRow>
                </React.Fragment>
              )
            })}
          </TableBody>
        </Table>
        <Separator />
        <div className="flex items-center justify-end space-x-4 py-2 mr-2">
          <Button
            variant="outline"
            size="sm"
            onClick={handlePreviousPage}
            disabled={currentPage === 1}
          >
            <HiChevronLeft className="h-4 w-4" />
          </Button>
          <div className="flex items-center gap-1">
            <span className="text-sm font-medium">{currentPage}</span>
          </div>
          <Button
            variant="outline"
            size="sm"
            onClick={handleNextPage}
            disabled={!data?.journalEntries?.pageInfo.hasNextPage}
          >
            <HiChevronRight className="h-4 w-4" />
          </Button>
        </div>
      </div>
    </JournalPageCard>
  )
}

export default JournalPage

const isFirstInGroup = (entries: JournalEntry[], index: number): boolean => {
  if (index === 0) return true
  return (
    entries[index - 1]?.ledgerTransaction.ledgerTransactionId !==
    entries[index].ledgerTransaction.ledgerTransactionId
  )
}
