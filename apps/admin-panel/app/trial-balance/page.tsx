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
import { useRouter } from "next/navigation"
import { useTranslations } from "next-intl"

import { Label } from "@lana/web/ui/label"

import { Separator } from "@lana/web/ui/separator"

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
      total {
        usd {
          ...UsdLedgerBalanceRangeFragment
        }
        btc {
          ...BtcLedgerBalanceRangeFragment
        }
      }
      accounts {
        ...TrialBalanceAccountBase
        children {
          ...TrialBalanceAccountBase
          children {
            ...TrialBalanceAccountBase
            children {
              ...TrialBalanceAccountBase
              children {
                ...TrialBalanceAccountBase
                children {
                  ...TrialBalanceAccountBase
                  children {
                    ...TrialBalanceAccountBase
                    children {
                      ...TrialBalanceAccountBase
                      children {
                        ...TrialBalanceAccountBase
                        children {
                          ...TrialBalanceAccountBase
                          children {
                            ...TrialBalanceAccountBase
                            children {
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

type Account = NonNullable<
  NonNullable<GetTrialBalanceQuery["trialBalance"]>["accounts"]
>[0]

const TrialBalanceFooter = ({
  total,
  currency,
  layer,
  t,
}: {
  total: NonNullable<GetTrialBalanceQuery["trialBalance"]>["total"]
  currency: Currency
  layer: TrialBalanceLayers
  t: (key: string) => string
}) => {
  const totalData =
    currency === "usd"
      ? {
          start: total.usd.usdStart[layer],
          diff: total.usd.usdDiff[layer],
          end: total.usd.usdEnd[layer],
        }
      : {
          start: total.btc.btcStart[layer],
          diff: total.btc.btcDiff[layer],
          end: total.btc.btcEnd[layer],
        }

  return (
    <TableFooter className="border-t-4">
      <TableRow>
        <TableCell className="font-bold text-sm">{t("totals")}</TableCell>
        <TableCell />
        <TableCell className="text-right">
          <Balance align="end" currency={currency} amount={totalData.start.net} />
        </TableCell>
        <TableCell className="text-right">
          <Balance align="end" currency={currency} amount={totalData.diff.debit} />
        </TableCell>
        <TableCell className="text-right">
          <Balance align="start" currency={currency} amount={totalData.diff.credit} />
        </TableCell>
        <TableCell className="text-right">
          <Balance align="end" currency={currency} amount={totalData.end.net} />
        </TableCell>
      </TableRow>
    </TableFooter>
  )
}

function TrialBalancePage() {
  const t = useTranslations("TrialBalance")

  const router = useRouter()

  const [dateRange, setDateRange] = useState<DateRange>(getInitialDateRange())
  const [currency, setCurrency] = useState<Currency>("usd")
  const [layer, setLayer] = useState<TrialBalanceLayers>("settled")

  const { data, loading, error } = useGetTrialBalanceQuery({
    variables: {
      from: dateRange.from,
      until: dateRange.until,
    },
  })
  const total = data?.trialBalance?.total
  const accounts = data?.trialBalance?.accounts

  const renderAccount = (account: Account, isRoot = false): React.ReactElement | null => {
    if (!shouldShowAccount(account)) return null
    const balanceData = getBalanceData(account.balanceRange, currency, layer)
    return (
      <React.Fragment key={account.id}>
        <TableRow
          className="cursor-pointer hover:bg-muted/50"
          onClick={() => router.push(`/ledger-accounts/${account.code}`)}
        >
          <TableCell>
            <div
              className={`font-mono text-xs  ${isRoot ? "font-bold" : "text-gray-500"}`}
            >
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
        {account.children?.map((child) => renderAccount(child as Account, false))}
      </React.Fragment>
    )
  }

  if (error) return <div className="text-destructive">{error.message}</div>
  if (loading && !data) return null
  if (!total) return <div>{t("noAccountsPresent")}</div>

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
        <div className="overflow-x-auto rounded-md border">
          <Table>
            <TableHeader className="bg-secondary [&_tr:hover]:!bg-secondary">
              <TableRow>
                <TableHead className="w-36 ">{t("table.headers.accountCode")}</TableHead>
                <TableHead className="w-40">{t("table.headers.accountName")}</TableHead>
                <TableHead className="text-right w-40">
                  {t("table.headers.beginningBalance")}
                </TableHead>
                <TableHead className="text-right w-48">
                  {t("table.headers.debits")}
                </TableHead>
                <TableHead className="text-left w-32">
                  {t("table.headers.credits")}
                </TableHead>
                <TableHead className="text-right w-32">
                  {t("table.headers.endingBalance")}
                </TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {accounts?.map((account) => renderAccount(account, true))}
            </TableBody>
            <TrialBalanceFooter total={total} currency={currency} layer={layer} t={t} />
          </Table>
        </div>
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

const shouldShowAccount = (account: Account): boolean =>
  Boolean(account.code && hasInEitherSettledOrPending(account.balanceRange))
