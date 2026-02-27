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

import { Label } from "@lana/web/ui/label"
import { Separator } from "@lana/web/ui/separator"
import { Skeleton } from "@lana/web/ui/skeleton"

import { Account } from "./account"
import { AsOfDateSelector, getInitialAsOfDate } from "./as-of-date-selector"

import { BalanceSheetQuery, useBalanceSheetQuery } from "@/lib/graphql/generated"
import Balance, { Currency } from "@/components/balance/balance"
import { CurrencySelection, LayerSelection, ReportLayer } from "@/components/report-filters/selectors"

type CategoryBalance = NonNullable<BalanceSheetQuery["balanceSheet"]>["assetsBalance"]

gql`
  query BalanceSheet($asOf: Date!) {
    balanceSheet(asOf: $asOf) {
      name
      assetsBalance {
        usd { settled { net } pending { net } }
        btc { settled { net } pending { net } }
      }
      liabilitiesBalance {
        usd { settled { net } pending { net } }
        btc { settled { net } pending { net } }
      }
      equityBalance {
        usd { settled { net } pending { net } }
        btc { settled { net } pending { net } }
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

export default function BalanceSheetPage() {
  const initialAsOf = useMemo(() => getInitialAsOfDate(), [])
  const [asOf, setAsOf] = useState<string>(initialAsOf)
  const handleDateChange = useCallback((newAsOf: string) => {
    setAsOf(newAsOf)
  }, [])

  const { data, loading, error } = useBalanceSheetQuery({
    variables: { asOf },
    fetchPolicy: "cache-and-network",
  })

  return (
    <>
      <BalanceSheetView
        data={data?.balanceSheet}
        loading={loading && !data}
        error={error}
        asOf={asOf}
        setAsOf={handleDateChange}
      />
    </>
  )
}

interface BalanceSheetViewProps {
  data?: BalanceSheetQuery["balanceSheet"]
  loading: boolean
  error: Error | undefined
  asOf: string
  setAsOf: (asOf: string) => void
}

const BalanceSheetView = ({
  data,
  loading,
  error,
  asOf,
  setAsOf,
}: BalanceSheetViewProps) => {
  const t = useTranslations("BalanceSheet")
  const [currency, setCurrency] = useState<Currency>("usd")
  const [layer, setLayer] = useState<ReportLayer>("settled")

  if (error) return <div className="text-destructive">{error.message}</div>

  const assets = data?.categories?.filter((cat) => cat.name === "Assets")
  const liabilities = data?.categories?.filter((cat) => cat.name === "Liabilities")
  const equity = data?.categories?.filter((cat) => cat.name === "Equity")

  const liabilitiesAndEquity = [...(liabilities || []), ...(equity || [])]

  const categoryBalanceMap: Record<string, CategoryBalance | undefined> = {
    Assets: data?.assetsBalance,
    Liabilities: data?.liabilitiesBalance,
    Equity: data?.equityBalance,
  }

  const assetsTotal = getCategoryBalanceNet(data?.assetsBalance, currency, layer)
  const liabilitiesAndEquityTotal =
    getCategoryBalanceNet(data?.liabilitiesBalance, currency, layer) +
    getCategoryBalanceNet(data?.equityBalance, currency, layer)

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("title")}</CardTitle>
        <CardDescription>{t("description")}</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="flex items-center rounded-md flex-wrap w-fit gap-2 mb-2">
          <div>
            <Label>{t("asOf")}</Label>
            <AsOfDateSelector asOf={asOf} onDateChange={setAsOf} />
          </div>
          <div className="flex items-center h-14">
            <Separator orientation="vertical" />
          </div>
          <div className="flex flex-col gap-2">
            <CurrencySelection currency={currency} setCurrency={setCurrency} />
          </div>
          <div className="flex items-center h-14">
            <Separator orientation="vertical" />
          </div>
          <div className="flex flex-col gap-2">
            <LayerSelection layer={layer} setLayer={setLayer} />
          </div>
        </div>

        {loading || !data ? (
          <Skeleton className="h-96 w-full" />
        ) : (
          <div className="flex justify-between border rounded-md">
            {assets && assets.length > 0 && (
              <BalanceSheetColumn
                title={t("columns.assets")}
                categories={assets}
                categoryBalanceMap={categoryBalanceMap}
                currency={currency}
                layer={layer}
                total={assetsTotal}
              />
            )}
            <div className="w-px min-h-full bg-border" />
            {liabilitiesAndEquity && liabilitiesAndEquity.length > 0 && (
              <BalanceSheetColumn
                title={t("columns.liabilitiesAndEquity")}
                categories={liabilitiesAndEquity}
                categoryBalanceMap={categoryBalanceMap}
                currency={currency}
                layer={layer}
                total={liabilitiesAndEquityTotal}
              />
            )}
          </div>
        )}
      </CardContent>
    </Card>
  )
}

interface BalanceSheetColumnProps {
  title: string
  categories: NonNullable<BalanceSheetQuery["balanceSheet"]>["categories"]
  categoryBalanceMap: Record<string, CategoryBalance | undefined>
  currency: Currency
  layer: ReportLayer
  total: number
}

function BalanceSheetColumn({
  title,
  categories,
  categoryBalanceMap,
  currency,
  layer,
  total,
}: BalanceSheetColumnProps) {
  return (
    <div className="grow flex flex-col justify-between w-1/2">
      <Table>
        <TableBody>
          {categories.map((category) => (
            <CategoryRow
              key={category.id}
              category={category}
              balance={categoryBalanceMap[category.name]}
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
  balance: CategoryBalance | undefined
  currency: Currency
  layer: ReportLayer
}

function CategoryRow({ category, balance, currency, layer }: CategoryRowProps) {
  const t = useTranslations("BalanceSheet")
  const categoryBalance = getCategoryBalanceNet(balance, currency, layer)

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

function getCategoryBalanceNet(
  balance: NonNullable<BalanceSheetQuery["balanceSheet"]>["assetsBalance"] | undefined,
  currency: Currency,
  layer: ReportLayer,
): number {
  if (!balance) return 0
  return balance[currency][layer].net
}
