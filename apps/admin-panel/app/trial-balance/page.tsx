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
        childrenWithCodeAndActivity(from: $from, until: $until) {
          ...TrialBalanceAccountBase
          childrenWithCodeAndActivity(from: $from, until: $until) {
            ...TrialBalanceAccountBase
            childrenWithCodeAndActivity(from: $from, until: $until) {
              ...TrialBalanceAccountBase
              childrenWithCodeAndActivity(from: $from, until: $until) {
                ...TrialBalanceAccountBase
                childrenWithCodeAndActivity(from: $from, until: $until) {
                  ...TrialBalanceAccountBase
                  childrenWithCodeAndActivity(from: $from, until: $until) {
                    ...TrialBalanceAccountBase
                    childrenWithCodeAndActivity(from: $from, until: $until) {
                      ...TrialBalanceAccountBase
                      childrenWithCodeAndActivity(from: $from, until: $until) {
                        ...TrialBalanceAccountBase
                        childrenWithCodeAndActivity(from: $from, until: $until) {
                          ...TrialBalanceAccountBase
                          childrenWithCodeAndActivity(from: $from, until: $until) {
                            ...TrialBalanceAccountBase
                            childrenWithCodeAndActivity(from: $from, until: $until) {
                              ...TrialBalanceAccountBase
                              childrenWithCodeAndActivity(from: $from, until: $until) {
                                ...TrialBalanceAccountBase
                              }
                            }
                          }
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  }

  fragment TrialBalanceAccountBase on LedgerAccount {
    id
    code
    name
    balanceRange {
      __typename
      ...UsdLedgerBalanceRangeFragment
      ...BtcLedgerBalanceRangeFragment
    }
  }
`

type Account = NonNullable<
  NonNullable<GetTrialBalanceQuery["trialBalance"]>["accounts"]
>[0]

const TrialBalanceAccountRow = ({
  account,
  isRoot,
  currency,
  layer,
}: {
  account: Account
  isRoot: boolean
  currency: Currency
  layer: ReportLayer
}) => {
  const router = useRouter()
  if (!hasInEitherSettledOrPending(account.balanceRange)) return null
  const balanceData = getBalanceData(account.balanceRange, currency, layer)

  return (
    <React.Fragment key={account.id}>
      <TableRow
        className="cursor-pointer hover:bg-muted/50"
        onClick={() => router.push(`/ledger-accounts/${account.code}`)}
      >
        <TableCell className="w-32">
          <div className={`font-mono text-xs ${isRoot ? "font-bold" : "text-gray-500"}`}>
            {account.code}
          </div>
        </TableCell>
        <TableCell className={`min-w-64 ${isRoot ? "font-bold" : ""}`}>
          {account.name}
        </TableCell>
        <TableCell className="text-right w-48">
          {balanceData?.start ? (
            <Balance
              align="end"
              currency={currency}
              className={isRoot ? "font-bold" : ""}
              amount={balanceData.start.net}
            />
          ) : (
            <span className="text-muted-foreground">-</span>
          )}
        </TableCell>
        <TableCell className="text-right w-48">
          {balanceData?.diff ? (
            <Balance
              align="end"
              currency={currency}
              className={isRoot ? "font-bold" : ""}
              amount={balanceData.diff.debit}
            />
          ) : (
            <span className="text-muted-foreground">-</span>
          )}
        </TableCell>
        <TableCell className="text-left w-48">
          {balanceData?.diff ? (
            <Balance
              align="start"
              currency={currency}
              className={isRoot ? "font-bold" : ""}
              amount={balanceData.diff.credit}
            />
          ) : (
            <span className="text-muted-foreground">-</span>
          )}
        </TableCell>
        <TableCell className="text-right w-48">
          {balanceData?.end ? (
            <Balance
              align="end"
              currency={currency}
              className={isRoot ? "font-bold" : ""}
              amount={balanceData.end.net}
            />
          ) : (
            <span className="text-muted-foreground">-</span>
          )}
        </TableCell>
      </TableRow>
      {account.childrenWithCodeAndActivity?.map((child) => (
        <TrialBalanceAccountRow
          key={child.id}
          account={child as Account}
          isRoot={false}
          currency={currency}
          layer={layer}
        />
      ))}
    </React.Fragment>
  )
}

function TrialBalancePage() {
  const t = useTranslations("TrialBalance")
  const [dateRange, setDateRange] = useState<DateRange>(getInitialDateRange())
  const [currency, setCurrency] = useState<Currency>("usd")
  const [layer, setLayer] = useState<ReportLayer>("settled")

  const { data, loading, error } = useGetTrialBalanceQuery({
    variables: {
      from: dateRange.from,
      until: dateRange.until,
    },
  })

  const accounts = data?.trialBalance?.accounts
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
        {loading && !accounts ? (
          <Skeleton className="h-96 w-full" />
        ) : (
          <div className="overflow-x-auto rounded-md border">
            <Table>
              <TableHeader className="bg-secondary [&_tr:hover]:!bg-secondary">
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
              {accounts ? (
                <TableBody>
                  {accounts.map((account) => (
                    <TrialBalanceAccountRow
                      key={account.id}
                      account={account}
                      isRoot={true}
                      currency={currency}
                      layer={layer}
                    />
                  ))}
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
