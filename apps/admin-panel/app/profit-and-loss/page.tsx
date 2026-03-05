"use client"
import { gql } from "@apollo/client"
import { useCallback, useState } from "react"

import { Table, TableBody, TableCell, TableFooter, TableRow } from "@lana/web/ui/table"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@lana/web/ui/card"
import { Skeleton } from "@lana/web/ui/skeleton"

import { useTranslations } from "next-intl"

import { Account, ProfitAndLossAccountNode } from "./account"

import {
  ProfitAndLossStatementQuery,
  useProfitAndLossStatementQuery,
} from "@/lib/graphql/generated"
import Balance, { Currency } from "@/components/balance/balance"
import { getYtdDateRange, DateRange } from "@/components/date-range-picker"
import { ReportFilters } from "@/components/report-filters"
import { ReportLayer } from "@/components/report-filters/selectors"

gql`
  fragment ProfitAndLossAccountFields on ProfitAndLossAccount {
    profitAndLossAccountId
    ledgerAccountId
    name
    code
    balanceRange {
      __typename
      ...UsdLedgerBalanceRangeFragment
      ...BtcLedgerBalanceRangeFragment
    }
  }

  query ProfitAndLossStatement($from: Date!, $until: Date) {
    profitAndLossStatement(from: $from, until: $until) {
      name
      total {
        usd {
          ...UsdLedgerBalanceRangeFragment
        }
        btc {
          ...BtcLedgerBalanceRangeFragment
        }
      }
      categories {
        ...ProfitAndLossAccountFields
        children {
          ...ProfitAndLossAccountFields
          children {
            ...ProfitAndLossAccountFields
            children {
              ...ProfitAndLossAccountFields
              children {
                ...ProfitAndLossAccountFields
              }
            }
          }
        }
      }
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
  }
`
interface ProfitAndLossProps {
  data?: ProfitAndLossStatementQuery["profitAndLossStatement"]
  loading: boolean
  error?: Error
  dateRange: DateRange
  setDateRange: (range: DateRange) => void
}

export default function ProfitAndLossStatementPage() {
  const [dateRange, setDateRange] = useState<DateRange>(getYtdDateRange)
  const handleDateChange = useCallback((newDateRange: DateRange) => {
    setDateRange(newDateRange)
  }, [])

  const { data, loading, error } = useProfitAndLossStatementQuery({
    variables: dateRange,
  })

  return (
    <ProfitAndLossStatement
      data={data?.profitAndLossStatement}
      loading={loading && !data}
      error={error}
      dateRange={dateRange}
      setDateRange={handleDateChange}
    />
  )
}

const ProfitAndLossStatement = ({
  data,
  loading,
  error,
  dateRange,
  setDateRange,
}: ProfitAndLossProps) => {
  const t = useTranslations("ProfitAndLoss")
  const [currency, setCurrency] = useState<Currency>("usd")
  const [layer, setLayer] = useState<ReportLayer>("settled")
  const [collapsedAccountIds, setCollapsedAccountIds] = useState<Set<string>>(new Set())
  const toggleCollapsedAccount = useCallback((accountId: string) => {
    setCollapsedAccountIds((prev) => {
      const next = new Set(prev)
      if (next.has(accountId)) {
        next.delete(accountId)
      } else {
        next.add(accountId)
      }
      return next
    })
  }, [])

  if (error) return <div className="text-destructive">{error.message}</div>

  const total = data?.total
  let netPeriod: number | undefined

  if (currency === "usd" && total?.usd) {
    netPeriod = total.usd.usdDiff[layer].net
  } else if (currency === "btc" && total?.btc) {
    netPeriod = total.btc.btcDiff[layer].net
  }

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
        {loading || !data?.categories || data.categories.length === 0 ? (
          <Skeleton className="h-96 w-full" />
        ) : (
          <div className="border rounded-md overflow-hidden">
            <Table>
              <TableBody>
                {data.categories.map((category) => {
                  let categoryPeriod: number | undefined
                  if (
                    category.balanceRange.__typename === "UsdLedgerAccountBalanceRange"
                  ) {
                    categoryPeriod = category.balanceRange.usdDiff[layer].net
                  } else if (
                    category.balanceRange.__typename === "BtcLedgerAccountBalanceRange"
                  ) {
                    categoryPeriod = category.balanceRange.btcDiff[layer].net
                  }
                  return (
                    <CategoryRow
                      key={category.profitAndLossAccountId}
                      category={category}
                      currency={currency}
                      layer={layer}
                      periodBalance={categoryPeriod}
                      collapsedAccountIds={collapsedAccountIds}
                      onToggleCollapsed={toggleCollapsedAccount}
                    />
                  )
                })}
              </TableBody>
              <TableFooter>
                <TableRow>
                  <TableCell className="uppercase font-bold">{t("net")}</TableCell>
                  <TableCell className="w-48">
                    <Balance
                      align="end"
                      currency={currency}
                      amount={netPeriod as CurrencyType}
                    />
                  </TableCell>
                </TableRow>
              </TableFooter>
            </Table>
          </div>
        )}
      </CardContent>
    </Card>
  )
}

interface CategoryRowProps {
  category: NonNullable<
    ProfitAndLossStatementQuery["profitAndLossStatement"]
  >["categories"][0]
  currency: Currency
  layer: ReportLayer
  periodBalance?: number
  collapsedAccountIds: Set<string>
  onToggleCollapsed: (accountId: string) => void
}

const CategoryRow = ({
  category,
  currency,
  layer,
  periodBalance,
  collapsedAccountIds,
  onToggleCollapsed,
}: CategoryRowProps) => {
  const t = useTranslations("ProfitAndLoss")

  return (
    <>
      <TableRow>
        <TableCell
          data-testid={`category-${category.name.toLowerCase()}`}
          className="flex items-center gap-2 text-primary font-semibold uppercase"
        >
          {t(`categories.${category.name.replace(/\s+/g, "")}`)}
        </TableCell>
        <TableCell className="w-48">
          <Balance
            align="end"
            currency={currency}
            amount={periodBalance as CurrencyType}
          />
        </TableCell>
      </TableRow>
      {category.children.map(
        (
          child: NonNullable<
            ProfitAndLossStatementQuery["profitAndLossStatement"]
          >["categories"][0]["children"][number],
        ) => (
          <Account
            key={child.profitAndLossAccountId}
            account={child as ProfitAndLossAccountNode}
            currency={currency}
            depth={1}
            layer={layer}
            collapsedAccountIds={collapsedAccountIds}
            onToggleCollapsed={onToggleCollapsed}
          />
        ),
      )}
    </>
  )
}
