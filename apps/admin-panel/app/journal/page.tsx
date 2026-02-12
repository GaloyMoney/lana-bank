"use client"

import React, { useMemo } from "react"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"
import Link from "next/link"
import { HiChevronLeft, HiChevronRight, HiInformationCircle } from "react-icons/hi"
import SimpleBar from "simplebar-react"

import { Card, CardDescription, CardHeader, CardTitle } from "@lana/web/ui/card"
import {
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@lana/web/ui/table"
import { Button } from "@lana/web/ui/button"
import { Separator } from "@lana/web/ui/separator"
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@lana/web/ui/tooltip"
import DateWithTooltip from "@lana/web/components/date-with-tooltip"
import { cn, formatDate } from "@lana/web/utils"

import { useJournalPagination } from "./use-journal-pagination"

import Balance from "@/components/balance/balance"
import { TableLoadingSkeleton } from "@/components/table-loading-skeleton"
import { TruncatedTextCell } from "@/app/components/truncated-text-cell"
import LayerLabel from "@/app/journal/layer-label"
import { DebitOrCredit, JournalEntriesQuery } from "@/lib/graphql/generated"

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

const getColumns = (
  t: ReturnType<typeof useTranslations>,
  tDesc: ReturnType<typeof useTranslations>,
) => [
  {
    key: "effective",
    label: t("table.effective"),
    width: "w-[10%]",
    align: "left",
    render: (entry: JournalEntry) =>
      formatDate(entry.ledgerTransaction.effective, { includeTime: false }),
  },
  {
    key: "createdAt",
    label: t("table.createdAt"),
    width: "w-[10%]",
    align: "left",
    render: (entry: JournalEntry) => <DateWithTooltip value={entry.createdAt} />,
  },
  {
    key: "description",
    label: t("table.description"),
    width: "w-[15%]",
    align: "left",
    render: (entry: JournalEntry) => {
      const raw = entry.ledgerTransaction.description || "-"
      const content = tDesc.has(raw) ? tDesc(raw) : raw
      return <TruncatedTextCell tooltipText={content}>{content}</TruncatedTextCell>
    },
  },
  {
    key: "TxId",
    label: "TxID",
    width: "w-[10%]",
    align: "left",
    render: (entry: JournalEntry) => {
      const txid = entry.ledgerTransaction.ledgerTransactionId
      const truncated = `${txid.slice(0, 5)}...${txid.slice(-5)}`
      return (
        <Tooltip>
          <TooltipTrigger asChild>
            <Link
              href={`/ledger-transactions/${txid}`}
              className="hover:underline text-sm"
            >
              {truncated}
            </Link>
          </TooltipTrigger>
          <TooltipContent>
            <p className="max-w-xs break-words">{txid}</p>
          </TooltipContent>
        </Tooltip>
      )
    },
  },
  {
    key: "entryType",
    label: t("table.entryType"),
    width: "w-[24%]",
    align: "left",
    render: (entry: JournalEntry) => (
      <TruncatedTextCell tooltipText={entry.entryType}>
        <span className="text-sm">{entry.entryType}</span>
      </TruncatedTextCell>
    ),
  },

  {
    key: "name",
    label: t("table.name"),
    width: "w-[15%]",
    align: "left",
    render: (entry: JournalEntry) => {
      const content = entry.ledgerAccount.name
      return (
        <TruncatedTextCell tooltipText={content}>
          <Link
            href={`/ledger-accounts/${entry.ledgerAccount.code || entry.ledgerAccount.ledgerAccountId}`}
            className="hover:underline"
          >
            {content}
          </Link>
        </TruncatedTextCell>
      )
    },
  },
  {
    key: "closestAccountWithCode",
    label: (
      <div className="flex items-center gap-1">
        {t("table.closestAccountWithCode")}
        <Tooltip>
          <TooltipTrigger asChild>
            <HiInformationCircle className="h-4 w-4 text-muted-foreground cursor-pointer" />
          </TooltipTrigger>
          <TooltipContent>
            <p>{t("table.closestAccountWithCodeTooltip")}</p>
          </TooltipContent>
        </Tooltip>
      </div>
    ),
    width: "w-[10%]",
    align: "left",
    render: (entry: JournalEntry) => {
      const code = entry.ledgerAccount.closestAccountWithCode?.code
      if (!code) return null
      return (
        <TruncatedTextCell tooltipText={code}>
          <Link href={`/ledger-accounts/${code}`} className="hover:underline">
            {code}
          </Link>
        </TruncatedTextCell>
      )
    },
  },
  {
    key: "layer",
    label: t("table.layer"),
    width: "w-[6%]",
    align: "left",
    render: (entry: JournalEntry) => <LayerLabel value={entry.layer} />,
  },
  {
    key: "debit",
    label: t("table.debit"),
    width: "w-[140px]",
    align: "right",
    className:
      "sticky right-[140px] z-10 bg-card group-hover:bg-muted transition-colors shadow-[inset_1px_0_0_0_hsl(var(--border))] border-b",
    headerClassName:
      "sticky right-[140px] z-30 bg-secondary shadow-[inset_1px_0_0_0_hsl(var(--border))]",
    render: (entry: JournalEntry) => {
      if (entry.direction !== DebitOrCredit.Debit) return null
      return entry.amount.__typename === "UsdAmount" ? (
        <Balance amount={entry.amount.usd} currency="usd" align="end" />
      ) : entry.amount.__typename === "BtcAmount" ? (
        <Balance amount={entry.amount.btc} currency="btc" align="end" />
      ) : null
    },
  },
  {
    key: "credit",
    label: t("table.credit"),
    width: "w-[140px]",
    align: "right",
    className:
      "sticky right-0 z-10 bg-card group-hover:bg-muted transition-colors border-b pr-6",
    headerClassName: "sticky right-0 z-30 bg-secondary pr-6",
    render: (entry: JournalEntry) => {
      if (entry.direction !== DebitOrCredit.Credit) return null
      return entry.amount.__typename === "UsdAmount" ? (
        <Balance amount={entry.amount.usd} currency="usd" align="end" />
      ) : entry.amount.__typename === "BtcAmount" ? (
        <Balance amount={entry.amount.btc} currency="btc" align="end" />
      ) : null
    },
  },
]

const JournalPageCard: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const t = useTranslations("Journal")
  return (
    <Card>
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
  const tDesc = useTranslations("TransactionDescriptions")
  const columns = useMemo(() => getColumns(t, tDesc), [t, tDesc])
  const scrollContainerRef = React.useRef<React.ComponentRef<typeof SimpleBar>>(null)

  const {
    loading,
    error,
    displayData,
    currentPage,
    hasNextPage,
    handleNextPage: nextPage,
    handlePreviousPage: prevPage,
    pageSize,
  } = useJournalPagination()

  const handleNextPage = async () => {
    await nextPage()
    const scrollElement = scrollContainerRef.current?.getScrollElement?.()
    scrollElement?.scrollTo({ top: 0, behavior: "smooth" })
  }

  const handlePreviousPage = () => {
    prevPage()
    const scrollElement = scrollContainerRef.current?.getScrollElement?.()
    scrollElement?.scrollTo({ top: 0, behavior: "smooth" })
  }

  if (loading) {
    return (
      <JournalPageCard>
        <TableLoadingSkeleton rows={pageSize} columns={columns.length} />
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

  if (displayData.length === 0) {
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
      <TooltipProvider>
        <div className="w-full">
          <SimpleBar
            ref={scrollContainerRef}
            style={{ maxHeight: "calc(100vh - 14rem)" }}
            autoHide={false}
            className="border rounded-md"
          >
            <table className="w-full caption-bottom text-sm table-fixed min-w-[85rem]">
              <TableHeader className="bg-secondary sticky top-0 z-20 [&_tr:hover]:!bg-secondary text-sm">
                <TableRow>
                  {columns.map((col) => (
                    <TableHead
                      key={col.key}
                      className={cn(
                        `${col.width} text-${col.align}`,
                        col.headerClassName,
                      )}
                    >
                      {col.label}
                    </TableHead>
                  ))}
                </TableRow>
              </TableHeader>
              <TableBody>
                {displayData.map((entry, index) => {
                  const isFirst = isFirstInGroup(displayData, index)
                  return (
                    <React.Fragment key={entry.entryId}>
                      {isFirst && index > 0 && (
                        <TableRow className="h-3">
                          <TableCell
                            colSpan={columns.length}
                            className="p-0 border-t-2 border-b-2 bg-muted/80"
                          />
                        </TableRow>
                      )}
                      <TableRow className="group hover:bg-muted">
                        {columns.map((col) => (
                          <TableCell
                            key={col.key}
                            className={cn(`text-${col.align}`, col.className)}
                          >
                            {col.render(entry)}
                          </TableCell>
                        ))}
                      </TableRow>
                    </React.Fragment>
                  )
                })}
              </TableBody>
            </table>
          </SimpleBar>
          <Separator />
        </div>
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
            disabled={!hasNextPage}
          >
            <HiChevronRight className="h-4 w-4" />
          </Button>
        </div>
      </TooltipProvider>
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
