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

import {
  ProfitAndLossStatementQuery,
  useProfitAndLossStatementQuery,
} from "@/lib/graphql/generated"
import Balance, { Currency } from "@/components/balance/balance"
import { getInitialDateRange, DateRange } from "@/components/date-range-picker"
import { ReportFilters } from "@/components/report-filters"
import { ReportLayer } from "@/components/report-filters/selectors"

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
interface ProfitAndLossProps {
  data?: ProfitAndLossStatementQuery["profitAndLossStatement"]
  loading: boolean
  error?: Error
  dateRange: DateRange
  setDateRange: (range: DateRange) => void
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
  const [layer, setLayer] = useState<ReportLayer>("settled")

  if (error) return <div className="text-destructive">{error.message}</div>

  const total = data?.total
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
                  let categoryEnd: number | undefined
                  if (
                    category.balanceRange.__typename === "UsdLedgerAccountBalanceRange"
                  ) {
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
