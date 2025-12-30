"use client"

import React, { useState } from "react"
import { gql } from "@apollo/client"

import {
  Table,
  TableBody,
  TableCell,
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
import { useRouter } from "next/navigation"
import { useTranslations } from "next-intl"

import { Skeleton } from "@lana/web/ui/skeleton"

import { GetTrialBalanceQuery, useGetTrialBalanceQuery } from "@/lib/graphql/generated"
import Balance, { Currency } from "@/components/balance/balance"
import { getInitialDateRange, DateRange } from "@/components/date-range-picker"
import { ReportFilters } from "@/components/report-filters"
import { ReportLayer } from "@/components/report-filters/selectors"

gql`
  query GetTrialBalance($from: Date!, $until: Date!) {
    trialBalance(from: $from, until: $until) {
      name
      accounts {
        ...TrialBalanceAccountBase
      }
    }
  }

  fragment TrialBalanceAccountBase on LedgerAccount {
    id
    code
    name
    isRootAccount
    balanceRange {
      __typename
      ...UsdLedgerBalanceRangeFragment
      ...BtcLedgerBalanceRangeFragment
    }
  }
`

type Account = NonNullable<
  NonNullable<GetTrialBalanceQuery["trialBalance"]>["accounts"]
>[number]

const TrialBalanceRowComponent = ({
  row,
  currency,
  layer,
  router,
}: {
  row: Account
  currency: Currency
  layer: ReportLayer
  router: ReturnType<typeof useRouter>
}) => {
  if (!hasInEitherSettledOrPending(row.balanceRange)) return null
  const balanceData = getBalanceData(row.balanceRange, currency, layer)

  return (
    <TableRow
      key={row.id}
      className="cursor-pointer hover:bg-muted/50"
      onClick={() => router.push(`/ledger-accounts/${row.code || row.id}`)}
    >
      <TableCell className="w-32">
        <div
          className={`font-mono text-xs  ${row.isRootAccount ? "font-bold" : "text-muted-foreground"}`}
        >
          {row.code}
        </div>
      </TableCell>
      <TableCell
        className={`min-w-64 ${row.isRootAccount ? "font-bold" : "text-muted-foreground"}`}
      >
        {row.name}
      </TableCell>
      <TableCell className={`text-right w-48 ${row.isRootAccount && "font-bold"}`}>
        {balanceData?.start ? (
          <Balance align="end" currency={currency} amount={balanceData.start.net} />
        ) : (
          <span>-</span>
        )}
      </TableCell>
      <TableCell className={`text-right w-48 ${row.isRootAccount && "font-bold"}`}>
        {balanceData?.diff ? (
          <Balance align="end" currency={currency} amount={balanceData.diff.debit} />
        ) : (
          <span>-</span>
        )}
      </TableCell>
      <TableCell className={`text-left w-48 ${row.isRootAccount && "font-bold"}`}>
        {balanceData?.diff ? (
          <Balance align="start" currency={currency} amount={balanceData.diff.credit} />
        ) : (
          <span>-</span>
        )}
      </TableCell>
      <TableCell className={`text-right w-48 ${row.isRootAccount && "font-bold"}`}>
        {balanceData?.end ? (
          <Balance align="end" currency={currency} amount={balanceData.end.net} />
        ) : (
          <span>-</span>
        )}
      </TableCell>
    </TableRow>
  )
}

function TrialBalancePage() {
  const t = useTranslations("TrialBalance")
  const [dateRange, setDateRange] = useState<DateRange>(getInitialDateRange())
  const [currency, setCurrency] = useState<Currency>("usd")
  const [layer, setLayer] = useState<ReportLayer>("settled")
  const router = useRouter()

  const {
    data: data,
    loading: loading,
    error: error,
  } = useGetTrialBalanceQuery({
    variables: {
      from: dateRange.from,
      until: dateRange.until,
    },
  })

  if (error) return <div className="text-destructive">{error.message}</div>

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("title")}</CardTitle>
        <CardDescription>{t("description")}</CardDescription>
      </CardHeader>
      <CardContent>
        <ReportFilters
          dateRange={dateRange}
          onDateChange={setDateRange}
          currency={currency}
          onCurrencyChange={setCurrency}
          layer={layer}
          onLayerChange={setLayer}
        />
        {loading && !data?.trialBalance?.accounts ? (
          <Skeleton className="h-96 w-full" />
        ) : (
          <div className="overflow-x-auto rounded-md border">
            <Table>
              <TableHeader className="bg-secondary [&_tr:hover]:bg-secondary!">
                <TableRow>
                  <TableHead className="w-32">{t("table.headers.accountCode")}</TableHead>
                  <TableHead className="min-w-64">
                    {t("table.headers.accountName")}
                  </TableHead>
                  <TableHead className="text-right w-48">
                    {t("table.headers.beginningBalance")}
                  </TableHead>
                  <TableHead className="text-right w-48">
                    {t("table.headers.debits")}
                  </TableHead>
                  <TableHead className="text-left w-48">
                    {t("table.headers.credits")}
                  </TableHead>
                  <TableHead className="text-right w-48">
                    {t("table.headers.endingBalance")}
                  </TableHead>
                </TableRow>
              </TableHeader>
              {data?.trialBalance?.accounts && data.trialBalance.accounts.length > 0 ? (
                <TableBody>
                  {data.trialBalance.accounts.map((entry) => {
                    return (
                      <TrialBalanceRowComponent
                        key={entry.id}
                        row={entry}
                        currency={currency}
                        layer={layer}
                        router={router}
                      />
                    )
                  })}
                </TableBody>
              ) : (
                <TableBody>
                  <TableRow>
                    <TableCell colSpan={6} className="text-center">
                      {t("noAccountsPresent")}
                    </TableCell>
                  </TableRow>
                </TableBody>
              )}
            </Table>
          </div>
        )}
      </CardContent>
    </Card>
  )
}

export default TrialBalancePage

const getBalanceData = (
  balanceRange: Account["balanceRange"],
  currency: Currency,
  layer: ReportLayer,
) => {
  if (!balanceRange) return null
  if (currency === "usd" && isUsdLedgerBalanceRange(balanceRange)) {
    return {
      start: balanceRange.usdStart?.[layer],
      diff: balanceRange.usdDiff?.[layer],
      end: balanceRange.usdEnd?.[layer],
    }
  }
  if (currency === "btc" && isBtcLedgerBalanceRange(balanceRange)) {
    return {
      start: balanceRange.btcStart?.[layer],
      diff: balanceRange.btcDiff?.[layer],
      end: balanceRange.btcEnd?.[layer],
    }
  }

  return null
}

const hasInEitherSettledOrPending = (balanceRange: Account["balanceRange"]): boolean => {
  if (!balanceRange) return false
  if (isUsdLedgerBalanceRange(balanceRange)) {
    return !!(
      balanceRange.usdStart?.settled?.net ||
      balanceRange.usdStart?.pending?.net ||
      balanceRange.usdDiff?.settled?.debit ||
      balanceRange.usdDiff?.settled?.credit ||
      balanceRange.usdDiff?.pending?.debit ||
      balanceRange.usdDiff?.pending?.credit ||
      balanceRange.usdEnd?.settled?.net ||
      balanceRange.usdEnd?.pending?.net
    )
  }
  if (isBtcLedgerBalanceRange(balanceRange)) {
    return !!(
      balanceRange.btcStart?.settled?.net ||
      balanceRange.btcStart?.pending?.net ||
      balanceRange.btcDiff?.settled?.debit ||
      balanceRange.btcDiff?.settled?.credit ||
      balanceRange.btcDiff?.pending?.debit ||
      balanceRange.btcDiff?.pending?.credit ||
      balanceRange.btcEnd?.settled?.net ||
      balanceRange.btcEnd?.pending?.net
    )
  }

  return false
}

const isUsdLedgerBalanceRange = (balanceRange: Account["balanceRange"]) =>
  balanceRange?.__typename === "UsdLedgerAccountBalanceRange"

const isBtcLedgerBalanceRange = (balanceRange: Account["balanceRange"]) =>
  balanceRange?.__typename === "BtcLedgerAccountBalanceRange"
