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

import { Label } from "@lana/web/ui/label"

import { Separator } from "@lana/web/ui/separator"
import { Skeleton } from "@lana/web/ui/skeleton"

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

gql`
  query GetTrialBalance($from: Date!, $until: Date!) {
    trialBalance(from: $from, until: $until) {
      name
      accounts {
        ...TrialBalanceAccountBase
        childrenWithCodeAndActivity {
          ...TrialBalanceAccountBase
          childrenWithCodeAndActivity {
            ...TrialBalanceAccountBase
            childrenWithCodeAndActivity {
              ...TrialBalanceAccountBase
              childrenWithCodeAndActivity {
                ...TrialBalanceAccountBase
                childrenWithCodeAndActivity {
                  ...TrialBalanceAccountBase
                  childrenWithCodeAndActivity {
                    ...TrialBalanceAccountBase
                    childrenWithCodeAndActivity {
                      ...TrialBalanceAccountBase
                      childrenWithCodeAndActivity {
                        ...TrialBalanceAccountBase
                        childrenWithCodeAndActivity {
                          ...TrialBalanceAccountBase
                          childrenWithCodeAndActivity {
                            ...TrialBalanceAccountBase
                            childrenWithCodeAndActivity {
                              ...TrialBalanceAccountBase
                              childrenWithCodeAndActivity {
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
  layer: TrialBalanceLayers
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
        <TableCell>
          <div className={`font-mono text-xs ${isRoot ? "font-bold" : "text-gray-500"}`}>
            {account.code}
          </div>
        </TableCell>
        <TableCell className={isRoot ? "font-bold" : ""}>{account.name}</TableCell>
        <TableCell className="text-right">
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
        <TableCell className="text-right">
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
        <TableCell className="text-left">
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
        <TableCell className="text-right">
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
  const [layer, setLayer] = useState<TrialBalanceLayers>("settled")

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
        <div className="mb-4 flex items-center">
          <div className="mr-8">
            <Label>{t("dateRange")}</Label>
            <DateRangeSelector initialDateRange={dateRange} onDateChange={setDateRange} />
          </div>
          <Separator orientation="vertical" className="h-14" />
          <div className="ml-2 mr-8">
            <TrialBalanceCurrencySelection
              currency={currency}
              setCurrency={setCurrency}
            />
          </div>
          <Separator orientation="vertical" className="h-14" />
          <div className="ml-2">
            <TrialBalanceLayerSelection layer={layer} setLayer={setLayer} />
          </div>
        </div>
        {loading && !accounts ? (
          <Skeleton className="h-96 w-full" />
        ) : (
          <div className="overflow-x-auto rounded-md border">
            <Table>
              <TableHeader className="bg-secondary [&_tr:hover]:!bg-secondary">
                <TableRow>
                  <TableHead className="w-28">{t("table.headers.accountCode")}</TableHead>
                  <TableHead className="w-56">{t("table.headers.accountName")}</TableHead>
                  <TableHead className="text-right w-40">
                    {t("table.headers.beginningBalance")}
                  </TableHead>
                  <TableHead className="text-right w-36">
                    {t("table.headers.debits")}
                  </TableHead>
                  <TableHead className="text-left w-24">
                    {t("table.headers.credits")}
                  </TableHead>
                  <TableHead className="text-right w-20">
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
  layer: TrialBalanceLayers,
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
