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

import { Account } from "./account"

import { PnlCurrencySelection, PnlLayerSelection } from "./pnl-currency-selector"

import {
  ProfitAndLossStatementQuery,
  useProfitAndLossStatementQuery,
} from "@/lib/graphql/generated"
import Balance, { Currency } from "@/components/balance/balance"

import {
  DateRange,
  DateRangeSelector,
  getInitialDateRange,
} from "@/components/date-range-picker"

gql`
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
        id
        name
        code
        balanceRange {
          __typename
          ...UsdLedgerBalanceRangeFragment
          ...BtcLedgerBalanceRangeFragment
        }
        children {
          id
          name
          code
          balanceRange {
            __typename
            ...UsdLedgerBalanceRangeFragment
            ...BtcLedgerBalanceRangeFragment
          }
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
    usdStart: start {
      ...UsdBalanceFragment
    }
    usdDiff: diff {
      ...UsdBalanceFragment
    }
    usdEnd: end {
      ...UsdBalanceFragment
    }
  }

  fragment BtcLedgerBalanceRangeFragment on BtcLedgerAccountBalanceRange {
    btcStart: start {
      ...BtcBalanceFragment
    }
    btcDiff: diff {
      ...BtcBalanceFragment
    }
    btcEnd: end {
      ...BtcBalanceFragment
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

const LoadingSkeleton = () => {
  return (
    <div className="space-y-6">
      <div className="space-y-4">
        <Skeleton className="h-10 w-72" />
        <Skeleton className="h-10 w-96" />
      </div>
      <Table>
        <TableBody>
          <TableRow>
            <TableCell>
              <Skeleton className="h-6 w-32" />
            </TableCell>
            <TableCell className="w-48">
              <Skeleton className="h-6 w-24 ml-auto" />
            </TableCell>
          </TableRow>
          {[1, 2, 3].map((i) => (
            <TableRow key={`revenue-${i}`}>
              <TableCell>
                <div className="flex gap-2">
                  <div className="w-6" />
                  <Skeleton className="h-5 w-48" />
                </div>
              </TableCell>
              <TableCell>
                <Skeleton className="h-5 w-24 ml-auto" />
              </TableCell>
            </TableRow>
          ))}
          <TableRow>
            <TableCell>
              <Skeleton className="h-6 w-32" />
            </TableCell>
            <TableCell className="w-48">
              <Skeleton className="h-6 w-24 ml-auto" />
            </TableCell>
          </TableRow>
        </TableBody>
        <TableFooter>
          <TableRow>
            <TableCell>
              <Skeleton className="h-6 w-32" />
            </TableCell>
            <TableCell>
              <Skeleton className="h-6 w-24 ml-auto" />
            </TableCell>
          </TableRow>
        </TableFooter>
      </Table>
    </div>
  )
}

export default function ProfitAndLossStatementPage() {
  const [dateRange, setDateRange] = useState<DateRange>(getInitialDateRange)
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
  const [layer, setLayer] = useState<PnlLayers>("settled")

  if (error) return <div className="text-destructive">{error.message}</div>
  if (loading || !data) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>{t("title")}</CardTitle>
          <CardDescription>{t("description")}</CardDescription>
        </CardHeader>
        <CardContent>
          <LoadingSkeleton />
        </CardContent>
      </Card>
    )
  }

  if (!data.categories || data.categories.length === 0) {
    return <div>No data available</div>
  }

  const total = data.total
  let netEnd: number | undefined

  if (currency === "usd" && total?.usd) {
    netEnd = total.usd.usdEnd[layer].net
  } else if (currency === "btc" && total?.btc) {
    netEnd = total.btc.btcEnd[layer].net
  }

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
        <PnlCurrencySelection currency={currency} setCurrency={setCurrency} />
        <PnlLayerSelection layer={layer} setLayer={setLayer} />
        <Table className="mt-6">
          <TableBody>
            {data.categories.map((category) => {
              let categoryEnd: number | undefined
              if (category.balanceRange.__typename === "UsdLedgerAccountBalanceRange") {
                categoryEnd = category.balanceRange.usdEnd[layer].net
              } else if (
                category.balanceRange.__typename === "BtcLedgerAccountBalanceRange"
              ) {
                categoryEnd = category.balanceRange.btcEnd[layer].net
              }
              return (
                <CategoryRow
                  key={category.id}
                  category={category}
                  currency={currency}
                  layer={layer}
                  endingBalance={categoryEnd}
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
                  amount={netEnd as CurrencyType}
                />
              </TableCell>
            </TableRow>
          </TableFooter>
        </Table>
      </CardContent>
    </Card>
  )
}

interface CategoryRowProps {
  category: NonNullable<
    ProfitAndLossStatementQuery["profitAndLossStatement"]
  >["categories"][0]
  currency: Currency
  layer: PnlLayers
  endingBalance?: number
}

const CategoryRow = ({ category, currency, layer, endingBalance }: CategoryRowProps) => {
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
            amount={endingBalance as CurrencyType}
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
            key={child.id}
            account={child}
            currency={currency}
            depth={1}
            layer={layer}
          />
        ),
      )}
    </>
  )
}
