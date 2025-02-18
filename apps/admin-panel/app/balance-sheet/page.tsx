"use client"
import { gql } from "@apollo/client"
import { useState, useCallback, useMemo } from "react"

import { Table, TableBody, TableCell, TableRow } from "@lana/web/ui/table"

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@lana/web/ui/card"

import { Skeleton } from "@lana/web/ui/skeleton"

import { Account } from "./account"

import {
  BalanceSheetQuery,
  StatementCategory,
  useBalanceSheetQuery,
} from "@/lib/graphql/generated"
import Balance, { Currency } from "@/components/balance/balance"
import { CurrencyLayerSelection } from "@/components/financial/currency-layer-selection"
import {
  DateRange,
  DateRangeSelector,
  getInitialDateRange,
} from "@/components/date-range-picker"

import { Satoshis, SignedSatoshis, SignedUsdCents, UsdCents } from "@/types"

gql`
  query BalanceSheet($from: Timestamp!, $until: Timestamp) {
    balanceSheet(from: $from, until: $until) {
      name
      balance {
        ...balancesByCurrency
      }
      categories {
        name
        amounts {
          ...balancesByCurrency
        }
        accounts {
          ... on Account {
            __typename
            id
            name
            amounts {
              ...balancesByCurrency
            }
          }
          ... on AccountSet {
            __typename
            id
            name

            amounts {
              ...balancesByCurrency
            }
          }
        }
      }
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

const BALANCE_FOR_CATEGORY: Record<string, { TransactionType: TransactionType }> = {
  Liabilities: { TransactionType: "netCredit" },
  Equity: { TransactionType: "netCredit" },
  Assets: { TransactionType: "netDebit" },
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

const BalanceSheet = ({
  data,
  loading,
  error,
  dateRange,
  setDateRange,
}: {
  data?: BalanceSheetQuery["balanceSheet"]
  loading: boolean
  error: Error | undefined
  dateRange: DateRange
  setDateRange: (dateRange: DateRange) => void
}) => {
  const [currency, setCurrency] = useState<Currency>("usd")
  const [layer, setLayer] = useState<Layers>("all")

  if (error) return <div className="text-destructive">{error.message}</div>

  if (loading && !data) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Balance Sheet</CardTitle>
          <CardDescription>
            A financial statement showing the company&apos;s assets, liabilities, and
            equity at a specific point in time.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <LoadingSkeleton />
        </CardContent>
      </Card>
    )
  }

  if (!data?.balance) return <div>No data</div>

  const assets = data.categories?.filter((category) => category.name === "Assets")

  const liabilitiesAndEquity = [
    data.categories?.find((category) => category.name === "Liabilities"),
    data.categories?.find((category) => category.name === "Equity"),
  ] as StatementCategory[]

  const totalLiabilitiesAndEquity = calculateTotalLiabilitiesAndEquity(
    liabilitiesAndEquity,
    currency,
    layer,
  )

  return (
    <Card>
      <CardHeader>
        <CardTitle>Balance Sheet</CardTitle>
        <CardDescription>
          A financial statement showing the company&apos;s assets, liabilities, and equity
          at a specific point in time.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div className="flex gap-6 items-center">
          <div>Date Range:</div>
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
              title="Total Assets"
              categories={assets}
              currency={currency}
              layer={layer}
              total={
                assets[0].amounts[currency].closingBalance[layer][
                  BALANCE_FOR_CATEGORY["Assets"].TransactionType
                ]
              }
            />
          )}
          <div className="w-0.5 min-h-full bg-secondary"></div>
          {liabilitiesAndEquity && liabilitiesAndEquity.length > 0 && (
            <BalanceSheetColumn
              title="Total Liabilities & Equity"
              categories={liabilitiesAndEquity}
              currency={currency}
              layer={layer}
              total={totalLiabilitiesAndEquity}
            />
          )}
        </div>
      </CardContent>
    </Card>
  )
}

function BalanceSheetHeader({
  currency,
  setCurrency,
  layer,
  setLayer,
}: {
  currency: Currency
  setCurrency: (currency: Currency) => void
  layer: Layers
  setLayer: (layer: Layers) => void
}) {
  return (
    <div>
      <CurrencyLayerSelection
        currency={currency}
        setCurrency={setCurrency}
        layer={layer}
        setLayer={setLayer}
      />
    </div>
  )
}

function BalanceSheetColumn({
  title,
  categories,
  currency,
  layer,
  total,
}: {
  title: string
  categories: StatementCategory[]
  currency: Currency
  layer: Layers
  total: number
}) {
  return (
    <div className="flex-grow flex flex-col justify-between w-1/2">
      <Table>
        <TableBody>
          {categories.map((category) => (
            <CategoryRow
              key={category.name}
              category={category}
              currency={currency}
              layer={layer}
              transactionType={BALANCE_FOR_CATEGORY[category.name].TransactionType}
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
                amount={total as Satoshis | SignedSatoshis | SignedUsdCents | UsdCents}
                className="font-semibold"
              />
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </div>
  )
}

function CategoryRow({
  category,
  currency,
  layer,
  transactionType,
}: {
  category: StatementCategory
  currency: Currency
  layer: Layers
  transactionType: TransactionType
}) {
  return (
    <>
      <TableRow className="bg-secondary">
        <TableCell
          className="flex items-center gap-2 text-primary font-semibold uppercase"
          data-testid={`category-name-${category.name.toLowerCase()}`}
        >
          {category.name}
        </TableCell>
        <TableCell className="w-48"></TableCell>
      </TableRow>
      {category.accounts.map((account) => (
        <Account
          key={account.id}
          account={account}
          currency={currency}
          layer={layer}
          transactionType={transactionType}
        />
      ))}
      {category.name !== "Assets" && (
        <TableRow>
          <TableCell className="flex items-center gap-2 text-textColor-secondary font-semibold uppercase text-xs">
            <div className="w-6" />
            Total
          </TableCell>
          <TableCell>
            <Balance
              align="end"
              className="font-semibold"
              currency={currency}
              amount={category.amounts[currency].closingBalance[layer][transactionType]}
            />
          </TableCell>
        </TableRow>
      )}
    </>
  )
}

function calculateTotalLiabilitiesAndEquity(
  categories: StatementCategory[] | undefined,
  currency: Currency,
  layer: Layers,
): number {
  return (
    categories?.reduce(
      (acc, category) =>
        acc +
        category.amounts[currency].closingBalance[layer][
          BALANCE_FOR_CATEGORY[category.name].TransactionType
        ],
      0,
    ) || 0
  )
}
