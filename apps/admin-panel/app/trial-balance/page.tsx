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

import { TrialBalanceAccount, useGetTrialBalanceQuery } from "@/lib/graphql/generated"

import Balance, { Currency } from "@/components/balance/balance"
import { CurrencyLayerSelection } from "@/components/financial/currency-layer-selection"
import {
  DateRange,
  DateRangeSelector,
  getInitialDateRange,
} from "@/components/date-range-picker"
import PaginatedTable, { Column, PaginatedData } from "@/components/paginated-table"

const DEFAULT_PAGESIZE = 15

gql`
  query GetTrialBalance(
    $from: Timestamp!
    $until: Timestamp!
    $first: Int!
    $after: String
  ) {
    trialBalance(from: $from, until: $until) {
      name
      total {
        ...balancesByCurrency
      }
      accounts(first: $first, after: $after) {
        edges {
          cursor
          node {
            id
            code
            name
            amounts {
              ...balancesByCurrency
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

  fragment balancesByCurrency on AccountAmountsByCurrency {
    btc: btc {
      ...rangedBtcBalances
    }
    usd: usd {
      ...rangedUsdBalances
    }
  }

  fragment rangedBtcBalances on BtcAccountAmountsInPeriod {
    openingBalance {
      ...btcBalances
    }
    closingBalance {
      ...btcBalances
    }
    amount {
      ...btcBalances
    }
  }

  fragment rangedUsdBalances on UsdAccountAmountsInPeriod {
    openingBalance {
      ...usdBalances
    }
    closingBalance {
      ...usdBalances
    }
    amount {
      ...usdBalances
    }
  }

  fragment btcBalances on LayeredBtcAccountBalanceAmounts {
    all {
      debit
      credit
      netDebit
      netCredit
    }
    settled {
      debit
      credit
      netDebit
      netCredit
    }
    pending {
      debit
      credit
      netDebit
      netCredit
    }
    encumbrance {
      debit
      credit
      netDebit
      netCredit
    }
  }

  fragment usdBalances on LayeredUsdAccountBalanceAmounts {
    all {
      debit
      credit
      netDebit
      netCredit
    }
    settled {
      debit
      credit
      netDebit
      netCredit
    }
    pending {
      debit
      credit
      netDebit
      netCredit
    }
    encumbrance {
      debit
      credit
      netDebit
      netCredit
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

type Layers = "all" | "settled" | "pending"

function TrialBalancePage() {
  const t = useTranslations("TrialBalance")
  const [dateRange, setDateRange] = useState<DateRange>(getInitialDateRange)

  const { data, loading, error, fetchMore } = useGetTrialBalanceQuery({
    variables: {
      from: dateRange.from,
      until: dateRange.until,
      first: DEFAULT_PAGESIZE,
    },
  })

  const [currency, setCurrency] = React.useState<Currency>("usd")
  const [layer, setLayer] = React.useState<Layers>("all")

  const router = useRouter()

  const total = data?.trialBalance.total
  const accounts = data?.trialBalance.accounts

  if (error) return <div className="text-destructive">{error.message}</div>
  if (loading && !data) {
    return <LoadingSkeleton />
  }
  if (!total) return <div>{t("noAccountsPresent")}</div>

  const Footer = (
    <TableFooter className="border-t-4">
      <TableRow>
        <TableCell className="font-bold">{t("totals")}</TableCell>
        <TableCell />
        <TableCell className="w-48">
          <Balance
            align="end"
            currency={currency}
            amount={total[currency].closingBalance[layer].debit}
          />
        </TableCell>
        <TableCell className="w-48">
          <Balance
            align="end"
            currency={currency}
            amount={total[currency].closingBalance[layer].credit}
          />
        </TableCell>
        <TableCell className="w-48">
          <Balance
            align="end"
            currency={currency}
            amount={total[currency].closingBalance[layer].netDebit}
          />
        </TableCell>
      </TableRow>
    </TableFooter>
  )

  const columns: Column<TrialBalanceAccount>[] = [
    {
      key: "code",
      label: t("table.headers.accountCode"),
      render: (code) => <div className="font-mono text-xs text-gray-500">{code}</div>,
    },
    {
      key: "name",
      label: t("table.headers.accountName"),
    },
    {
      key: "amounts",
      label: t("table.headers.debit"),
      labelClassName: "!justify-end",
      render: (amounts) => (
        <Balance
          align="end"
          currency={currency}
          amount={amounts[currency].closingBalance[layer].debit}
        />
      ),
    },
    {
      key: "amounts",
      label: t("table.headers.credit"),
      labelClassName: "!justify-end",
      render: (amounts) => (
        <Balance
          align="end"
          currency={currency}
          amount={amounts[currency].closingBalance[layer].credit}
        />
      ),
    },
    {
      key: "amounts",
      labelClassName: "!justify-end",
      label: t("table.headers.net"),
      render: (amounts) => (
        <Balance
          align="end"
          currency={currency}
          amount={amounts[currency].closingBalance[layer].netDebit}
        />
      ),
    },
  ]

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
        <CurrencyLayerSelection
          currency={currency}
          setCurrency={setCurrency}
          layer={layer}
          setLayer={setLayer}
        />
        <PaginatedTable<TrialBalanceAccount>
          columns={columns}
          data={accounts as PaginatedData<TrialBalanceAccount>}
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
          onClick={(account) => router.push(`/ledger-account/${account.code}`)}
          noDataText={t("noAccountsPresent")}
        />
        <div className="mt-4" />
      </CardContent>
    </Card>
  )
}

export default TrialBalancePage
