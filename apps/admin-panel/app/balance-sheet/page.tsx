"use client"
import { gql } from "@apollo/client"
import { useState, useCallback, useMemo } from "react"
import { useTranslations } from "next-intl"

import { Table, TableBody, TableCell, TableRow } from "@lana/web/ui/table"

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@lana/web/ui/card"

import { Skeleton } from "@lana/web/ui/skeleton"

import {
  PnlCurrencySelection,
  PnlLayerSelection,
} from "../profit-and-loss/pnl-currency-selector"

import { Account } from "./account"

import { BalanceSheetQuery, useBalanceSheetQuery } from "@/lib/graphql/generated"
import Balance, { Currency } from "@/components/balance/balance"

import {
  DateRange,
  DateRangeSelector,
  getInitialDateRange,
} from "@/components/date-range-picker"

gql`
  query BalanceSheet($from: Date!, $until: Date) {
    balanceSheet(from: $from, until: $until) {
      name
      balance {
        __typename
        ...UsdLedgerBalanceRangeFragment
        ...BtcLedgerBalanceRangeFragment
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

const LoadingSkeleton = () => {
  return (
    <div className="space-y-6">
      <div className="space-y-4">
        <Skeleton className="h-10 w-72" />
        <Skeleton className="h-10 w-96" />
      </div>
      <div className="flex gap-4 justify-between">
        <div className="w-1/2 space-y-4">
          {[1, 2, 3, 4, 5].map((i) => (
            <Skeleton key={`left-${i}`} className="h-16 w-full" />
          ))}
        </div>
        <div className="w-0.5 min-h-full bg-secondary" />
        <div className="w-1/2 space-y-4">
          {[1, 2, 3, 4, 5].map((i) => (
            <Skeleton key={`right-${i}`} className="h-16 w-full" />
          ))}
        </div>
      </div>
    </div>
  )
}

export default function BalanceSheetPage() {
  const initialDateRange = useMemo(() => getInitialDateRange(), [])
  const [dateRange, setDateRange] = useState<DateRange>(initialDateRange)
  const handleDateChange = useCallback((newDateRange: DateRange) => {
    setDateRange(newDateRange)
  }, [])

  const { data, loading, error } = useBalanceSheetQuery({
    variables: dateRange,
    fetchPolicy: "cache-and-network",
  })

  return (
    <>
      <BalanceSheet
        data={data?.balanceSheet}
        loading={loading && !data}
        error={error}
        dateRange={dateRange}
        setDateRange={handleDateChange}
      />
    </>
  )
}

interface BalanceSheetProps {
  data?: BalanceSheetQuery["balanceSheet"]
  loading: boolean
  error: Error | undefined
  dateRange: DateRange
  setDateRange: (dateRange: DateRange) => void
}

const BalanceSheet = ({
  data,
  loading,
  error,
  dateRange,
  setDateRange,
}: BalanceSheetProps) => {
  const t = useTranslations("BalanceSheet")
  const [currency, setCurrency] = useState<Currency>("usd")
  const [layer, setLayer] = useState<BalanceSheetLayers>("settled")

  if (error) return <div className="text-destructive">{error.message}</div>

  if (loading && !data) {
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

  if (!data?.balance) return <div>{t("noData")}</div>

  const assets = data.categories?.filter((cat) => cat.name === "Assets")
  const liabilities = data.categories?.filter((cat) => cat.name === "Liabilities")
  const equity = data.categories?.filter((cat) => cat.name === "Equity")

  const assetsTotal = getBalanceTotal(assets, currency, layer)

  const liabilitiesAndEquity = [...(liabilities || []), ...(equity || [])]
  const liabilitiesAndEquityTotal = getBalanceTotal(liabilitiesAndEquity, currency, layer)

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

        <BalanceSheetHeader
          currency={currency}
          setCurrency={setCurrency}
          layer={layer}
          setLayer={setLayer}
        />

        <div className="flex gap-4 justify-between mt-6">
          {assets && assets.length > 0 && (
            <BalanceSheetColumn
              title={t("columns.assets")}
              categories={assets}
              currency={currency}
              layer={layer}
              total={assetsTotal}
            />
          )}
          <div className="w-0.5 min-h-full bg-secondary"></div>
          {liabilitiesAndEquity && liabilitiesAndEquity.length > 0 && (
            <BalanceSheetColumn
              title={t("columns.liabilitiesAndEquity")}
              categories={liabilitiesAndEquity}
              currency={currency}
              layer={layer}
              total={liabilitiesAndEquityTotal}
            />
          )}
        </div>
      </CardContent>
    </Card>
  )
}

interface BalanceSheetHeaderProps {
  currency: Currency
  setCurrency: (currency: Currency) => void
  layer: BalanceSheetLayers
  setLayer: (layer: BalanceSheetLayers) => void
}

function BalanceSheetHeader({
  currency,
  setCurrency,
  layer,
  setLayer,
}: BalanceSheetHeaderProps) {
  return (
    <div>
      {/* TODO: update this to use common component */}
      <PnlCurrencySelection currency={currency} setCurrency={setCurrency} />
      <PnlLayerSelection layer={layer} setLayer={setLayer} />
    </div>
  )
}

interface BalanceSheetColumnProps {
  title: string
  categories: NonNullable<BalanceSheetQuery["balanceSheet"]>["categories"]
  currency: Currency
  layer: BalanceSheetLayers
  total: number
}

function BalanceSheetColumn({
  title,
  categories,
  currency,
  layer,
  total,
}: BalanceSheetColumnProps) {
  return (
    <div className="flex-grow flex flex-col justify-between w-1/2">
      <Table>
        <TableBody>
          {categories.map((category) => (
            <CategoryRow
              key={category.id}
              category={category}
              currency={currency}
              layer={layer}
            />
          ))}
        </TableBody>
      </Table>
      <Table>
        <TableBody>
          <TableRow className="bg-secondary">
            <TableCell className="uppercase font-bold">{title}</TableCell>
            <TableCell className="flex flex-col gap-2 items-end text-right font-semibold">
              <Balance
                align="end"
                currency={currency}
                amount={total as CurrencyType}
                className="font-semibold"
              />
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </div>
  )
}

interface CategoryRowProps {
  category: NonNullable<BalanceSheetQuery["balanceSheet"]>["categories"][0]
  currency: Currency
  layer: BalanceSheetLayers
}

function CategoryRow({ category, currency, layer }: CategoryRowProps) {
  const t = useTranslations("BalanceSheet")
  const categoryBalance = getBalance(category, currency, layer)

  return (
    <>
      <TableRow className="bg-secondary">
        <TableCell
          className="flex items-center gap-2 text-primary font-semibold uppercase"
          data-testid={`category-name-${category.name.toLowerCase()}`}
        >
          {t(`categories.${category.name.replace(/\s+/g, "")}`)}
        </TableCell>
        <TableCell className="w-48"></TableCell>
      </TableRow>
      {category.children?.map((child) => (
        <Account key={child.id} account={child} currency={currency} layer={layer} />
      ))}
      <TableRow>
        <TableCell className="flex items-center gap-2 text-textColor-secondary font-semibold uppercase text-xs">
          <div className="w-6" />
          {t("total")}
        </TableCell>
        <TableCell>
          <Balance
            align="end"
            className="font-semibold"
            currency={currency}
            amount={categoryBalance as CurrencyType}
          />
        </TableCell>
      </TableRow>
    </>
  )
}

function getBalance(
  item: NonNullable<BalanceSheetQuery["balanceSheet"]>["categories"][0],
  currency: Currency,
  layer: BalanceSheetLayers,
): number {
  if (!item.balanceRange) return 0
  if (
    currency === "usd" &&
    item.balanceRange.__typename === "UsdLedgerAccountBalanceRange"
  ) {
    return item.balanceRange.usdEnd[layer].net
  } else if (
    currency === "btc" &&
    item.balanceRange.__typename === "BtcLedgerAccountBalanceRange"
  ) {
    return item.balanceRange.btcEnd[layer].net
  }

  return 0
}

function getBalanceTotal(
  categories: NonNullable<BalanceSheetQuery["balanceSheet"]>["categories"] | undefined,
  currency: Currency,
  layer: BalanceSheetLayers,
): number {
  if (!categories || categories.length === 0) return 0
  return categories.reduce((total, category) => {
    return total + getBalance(category, currency, layer)
  }, 0)
}
