"use client"
import React, { useState } from "react"
import { gql } from "@apollo/client"

import {
  Table,
  TableBody,
  TableCell,
  TableFooter,
  TableHead,
  TableHeader,
  TableRow,
} from "@lana/web/ui/table"

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@lana/web/ui/card"

import { Skeleton } from "@lana/web/ui/skeleton"

import { useRouter } from "next/navigation"
import { useTranslations } from "next-intl"

import {
  TrialBalanceCurrencySelection,
  TrialBalanceLayerSelection,
  TrialBalanceLayers,
} from "./trial-balance-currency-selector"

import { GetTrialBalanceQuery, useGetTrialBalanceQuery } from "@/lib/graphql/generated"
import Balance, { Currency } from "@/components/balance/balance"
import {
  DateRange,
  DateRangeSelector,
  getInitialDateRange,
} from "@/components/date-range-picker"
import PaginatedTable, { Column, PaginatedData } from "@/components/paginated-table"

const DEFAULT_PAGESIZE = 15

gql`
  query GetTrialBalance($from: Date!, $until: Date!, $first: Int!, $after: String) {
    trialBalance(from: $from, until: $until) {
      name
      total {
        usd {
          ...UsdLedgerBalanceRangeFragment
        }
        btc {
          ...BtcLedgerBalanceRangeFragment
        }
      }
      accounts(first: $first, after: $after) {
        edges {
          cursor
          node {
            id
            code
            name
            balanceRange {
              __typename
              ...UsdLedgerBalanceRangeFragment
              ...BtcLedgerBalanceRangeFragment
            }
            children {
              id
              code
              name
              balanceRange {
                __typename
                ...UsdLedgerBalanceRangeFragment
                ...BtcLedgerBalanceRangeFragment
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
  }

  fragment UsdBalanceFragment on UsdLedgerAccountBalance {
    settled {
      debit
      credit
      net
    }
    pending {
      debit
      credit
      net
    }
    encumbrance {
      debit
      credit
      net
    }
  }

  fragment BtcBalanceFragment on BtcLedgerAccountBalance {
    settled {
      debit
      credit
      net
    }
    pending {
      debit
      credit
      net
    }
    encumbrance {
      debit
      credit
      net
    }
  }

  fragment UsdLedgerBalanceRangeFragment on UsdLedgerAccountBalanceRange {
    usdStart: open {
      ...UsdBalanceFragment
    }
    usdDiff: periodActivity {
      ...UsdBalanceFragment
    }
    usdEnd: close {
      ...UsdBalanceFragment
    }
  }

  fragment BtcLedgerBalanceRangeFragment on BtcLedgerAccountBalanceRange {
    btcStart: open {
      ...BtcBalanceFragment
    }
    btcDiff: periodActivity {
      ...BtcBalanceFragment
    }
    btcEnd: close {
      ...BtcBalanceFragment
    }
  }
`

const LoadingSkeleton = () => {
  return (
    <div className="space-y-4" data-testid="loading-skeleton">
      <div className="space-y-4">
        <Skeleton className="h-10 w-72" />
        <Skeleton className="h-10 w-96" />
      </div>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>
              <Skeleton className="h-5 w-32" />
            </TableHead>
            <TableHead>
              <Skeleton className="h-5 w-24 ml-auto" />
            </TableHead>
            <TableHead>
              <Skeleton className="h-5 w-24 ml-auto" />
            </TableHead>
            <TableHead>
              <Skeleton className="h-5 w-24 ml-auto" />
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {[1, 2, 3, 4, 5].map((i) => (
            <TableRow key={i}>
              <TableCell>
                <Skeleton className="h-5 w-full" />
              </TableCell>
              <TableCell>
                <Skeleton className="h-5 w-24 ml-auto" />
              </TableCell>
              <TableCell>
                <Skeleton className="h-5 w-24 ml-auto" />
              </TableCell>
              <TableCell>
                <Skeleton className="h-5 w-24 ml-auto" />
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  )
}

type TrialBalanceAccountNode = NonNullable<
  NonNullable<GetTrialBalanceQuery["trialBalance"]>["accounts"]
>["edges"][0]["node"]

type BalanceRange = NonNullable<TrialBalanceAccountNode["balanceRange"]>

function isUsdLedgerBalanceRange(balanceRange: BalanceRange | null | undefined) {
  return !!balanceRange && balanceRange.__typename === "UsdLedgerAccountBalanceRange"
}

function isBtcLedgerBalanceRange(balanceRange: BalanceRange | null | undefined) {
  return !!balanceRange && balanceRange.__typename === "BtcLedgerAccountBalanceRange"
}

const hasNonZeroBalanceRangeData = (balanceRange: BalanceRange) => {
  if (
    balanceRange.__typename === "UsdLedgerAccountBalanceRange" &&
    (balanceRange.usdStart.pending.debit ||
      balanceRange.usdStart.pending.credit ||
      balanceRange.usdStart.pending.net ||
      balanceRange.usdStart.settled.debit ||
      balanceRange.usdStart.settled.credit ||
      balanceRange.usdStart.settled.net ||
      balanceRange.usdStart.encumbrance.debit ||
      balanceRange.usdStart.encumbrance.credit ||
      balanceRange.usdStart.encumbrance.net ||
      balanceRange.usdDiff.pending.debit ||
      balanceRange.usdDiff.pending.credit ||
      balanceRange.usdDiff.pending.net ||
      balanceRange.usdDiff.settled.debit ||
      balanceRange.usdDiff.settled.credit ||
      balanceRange.usdDiff.settled.net ||
      balanceRange.usdDiff.encumbrance.debit ||
      balanceRange.usdDiff.encumbrance.credit ||
      balanceRange.usdDiff.encumbrance.net ||
      balanceRange.usdEnd.pending.debit ||
      balanceRange.usdEnd.pending.credit ||
      balanceRange.usdEnd.pending.net ||
      balanceRange.usdEnd.settled.debit ||
      balanceRange.usdEnd.settled.credit ||
      balanceRange.usdEnd.settled.net ||
      balanceRange.usdEnd.encumbrance.debit ||
      balanceRange.usdEnd.encumbrance.credit ||
      balanceRange.usdEnd.encumbrance.net)
  )
    return true
  if (
    balanceRange.__typename === "BtcLedgerAccountBalanceRange" &&
    (balanceRange.btcStart.pending.debit ||
      balanceRange.btcStart.pending.credit ||
      balanceRange.btcStart.pending.net ||
      balanceRange.btcStart.settled.debit ||
      balanceRange.btcStart.settled.credit ||
      balanceRange.btcStart.settled.net ||
      balanceRange.btcStart.encumbrance.debit ||
      balanceRange.btcStart.encumbrance.credit ||
      balanceRange.btcStart.encumbrance.net ||
      balanceRange.btcDiff.pending.debit ||
      balanceRange.btcDiff.pending.credit ||
      balanceRange.btcDiff.pending.net ||
      balanceRange.btcDiff.settled.debit ||
      balanceRange.btcDiff.settled.credit ||
      balanceRange.btcDiff.settled.net ||
      balanceRange.btcDiff.encumbrance.debit ||
      balanceRange.btcDiff.encumbrance.credit ||
      balanceRange.btcDiff.encumbrance.net ||
      balanceRange.btcEnd.pending.debit ||
      balanceRange.btcEnd.pending.credit ||
      balanceRange.btcEnd.pending.net ||
      balanceRange.btcEnd.settled.debit ||
      balanceRange.btcEnd.settled.credit ||
      balanceRange.btcEnd.settled.net ||
      balanceRange.btcEnd.encumbrance.debit ||
      balanceRange.btcEnd.encumbrance.credit ||
      balanceRange.btcEnd.encumbrance.net)
  )
    return true
  return false
}

function TrialBalancePage() {
  const t = useTranslations("TrialBalance")
  const [dateRange, setDateRange] = useState<DateRange>(getInitialDateRange())

  const { data, loading, error, fetchMore } = useGetTrialBalanceQuery({
    variables: {
      from: dateRange.from,
      until: dateRange.until,
      first: DEFAULT_PAGESIZE,
    },
  })

  const [currency, setCurrency] = React.useState<Currency>("usd")
  const [layer, setLayer] = React.useState<TrialBalanceLayers>("settled")

  const router = useRouter()

  const total = data?.trialBalance.total
  const accounts = data?.trialBalance.accounts

  if (error) return <div className="text-destructive">{error.message}</div>
  if (loading && !data) {
    return <LoadingSkeleton />
  }
  if (!total) return <div>{t("noAccountsPresent")}</div>

  const columns: Column<TrialBalanceAccountNode>[] = [
    {
      key: "code",
      label: t("table.headers.accountCode"),
      render: (code: string) => (
        <div className="font-mono text-xs text-gray-500">{code}</div>
      ),
    },
    {
      key: "name",
      label: t("table.headers.accountName"),
    },
    {
      key: "balanceRange",
      label: t("table.headers.beginningBalance"),
      labelClassName: "!justify-end",
      render: (_, node) => {
        if (currency === "usd" && isUsdLedgerBalanceRange(node.balanceRange)) {
          return (
            <Balance
              align="end"
              currency={currency}
              amount={node.balanceRange.usdStart[layer].net}
            />
          )
        } else if (currency === "btc" && isBtcLedgerBalanceRange(node.balanceRange)) {
          return (
            <Balance
              align="end"
              currency={currency}
              amount={node.balanceRange.btcStart[layer].net}
            />
          )
        }
        return null
      },
    },
    {
      key: "balanceRange",
      label: t("table.headers.debits"),
      labelClassName: "!justify-end",
      render: (_, node) => {
        if (currency === "usd" && isUsdLedgerBalanceRange(node.balanceRange)) {
          return (
            <Balance
              align="end"
              currency={currency}
              amount={node.balanceRange.usdDiff[layer].debit}
            />
          )
        } else if (currency === "btc" && isBtcLedgerBalanceRange(node.balanceRange)) {
          return (
            <Balance
              align="end"
              currency={currency}
              amount={node.balanceRange.btcDiff[layer].debit}
            />
          )
        }
        return null
      },
    },
    {
      key: "balanceRange",
      label: t("table.headers.credits"),
      labelClassName: "!justify-end",
      render: (_, node) => {
        if (currency === "usd" && isUsdLedgerBalanceRange(node.balanceRange)) {
          return (
            <Balance
              align="end"
              currency={currency}
              amount={node.balanceRange.usdDiff[layer].credit}
            />
          )
        } else if (currency === "btc" && isBtcLedgerBalanceRange(node.balanceRange)) {
          return (
            <Balance
              align="end"
              currency={currency}
              amount={node.balanceRange.btcDiff[layer].credit}
            />
          )
        }
        return null
      },
    },
    {
      key: "balanceRange",
      label: t("table.headers.endingBalance"),
      labelClassName: "!justify-end",
      render: (_, node) => {
        if (currency === "usd" && isUsdLedgerBalanceRange(node.balanceRange)) {
          return (
            <Balance
              align="end"
              currency={currency}
              amount={node.balanceRange.usdEnd[layer].net}
            />
          )
        } else if (currency === "btc" && isBtcLedgerBalanceRange(node.balanceRange)) {
          return (
            <Balance
              align="end"
              currency={currency}
              amount={node.balanceRange.btcEnd[layer].net}
            />
          )
        }
        return null
      },
    },
  ]

  const Footer = (
    <TableFooter className="border-t-4">
      <TableRow>
        <TableCell className="font-bold">{t("totals")}</TableCell>
        <TableCell />
        <TableCell className="w-48">
          <Balance
            align="end"
            currency={currency}
            amount={
              currency === "usd"
                ? total.usd.usdStart[layer].net
                : total.btc.btcStart[layer].net
            }
          />
        </TableCell>
        <TableCell className="w-48">
          <Balance
            align="end"
            currency={currency}
            amount={
              currency === "usd"
                ? total.usd.usdDiff[layer].debit
                : total.btc.btcDiff[layer].debit
            }
          />
        </TableCell>
        <TableCell className="w-48">
          <Balance
            align="end"
            currency={currency}
            amount={
              currency === "usd"
                ? total.usd.usdDiff[layer].credit
                : total.btc.btcDiff[layer].credit
            }
          />
        </TableCell>
        <TableCell className="w-48">
          <Balance
            align="end"
            currency={currency}
            amount={
              currency === "usd"
                ? total.usd.usdEnd[layer].net
                : total.btc.btcEnd[layer].net
            }
          />
        </TableCell>
      </TableRow>
    </TableFooter>
  )

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("title")}</CardTitle>
        <CardDescription>{t("description")}</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="flex gap-6 items-center">
          <div>{t("dateRange")}:</div>
          <DateRangeSelector initialDateRange={dateRange} onDateChange={setDateRange} />
        </div>
        <div>
          <TrialBalanceCurrencySelection currency={currency} setCurrency={setCurrency} />
          <TrialBalanceLayerSelection layer={layer} setLayer={setLayer} />
        </div>
        <PaginatedTable<TrialBalanceAccountNode>
          columns={columns}
          data={accounts as PaginatedData<TrialBalanceAccountNode>}
          loading={loading}
          pageSize={DEFAULT_PAGESIZE}
          fetchMore={async (cursor) =>
            fetchMore({
              variables: {
                after: cursor,
                from: dateRange.from,
                until: dateRange.until,
                first: DEFAULT_PAGESIZE,
              },
            })
          }
          customFooter={Footer}
          style="compact"
          onClick={(account) =>
            router.push(`/ledger-account/${account.code || account.id}`)
          }
          noDataText={t("noAccountsPresent")}
          subRows={(account) =>
            account.children.filter((child) =>
              hasNonZeroBalanceRangeData(child.balanceRange),
            ) as TrialBalanceAccountNode[]
          }
        />
        <div className="mt-4" />
      </CardContent>
    </Card>
  )
}

export default TrialBalancePage
