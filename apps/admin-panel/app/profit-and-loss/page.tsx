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

import { Account } from "./account"

import {
  ProfitAndLossStatementQuery,
  StatementCategory,
  useProfitAndLossStatementQuery,
} from "@/lib/graphql/generated"
import Balance, { Currency } from "@/components/balance/balance"

import { CurrencyLayerSelection } from "@/components/financial/currency-layer-selection"
import {
  DateRange,
  DateRangeSelector,
  getInitialDateRange,
} from "@/components/date-range-picker"

gql`
  query ProfitAndLossStatement($from: Timestamp!, $until: Timestamp) {
    profitAndLossStatement(from: $from, until: $until) {
      name
      net {
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

const BALANCE_FOR_CATEGORY: {
  [key: string]: { TransactionType: TransactionType }
} = {
  Revenue: { TransactionType: "netCredit" },
  Expenses: { TransactionType: "netDebit" },
}

export default function ProfitAndLossStatementPage() {
  const [dateRange, setDateRange] = useState<DateRange>(getInitialDateRange)
  const handleDateChange = useCallback((newDateRange: DateRange) => {
    setDateRange(newDateRange)
  }, [])

  const {
    data: ProfitAndLossStatementData,
    loading: ProfitAndLossStatementLoading,
    error: ProfitAndLossStatementError,
  } = useProfitAndLossStatementQuery({
    variables: dateRange,
  })

  return (
    <ProfitAndLossStatement
      data={ProfitAndLossStatementData?.profitAndLossStatement}
      loading={ProfitAndLossStatementLoading && !ProfitAndLossStatementData}
      error={ProfitAndLossStatementError}
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
}: {
  data?: ProfitAndLossStatementQuery["profitAndLossStatement"]
  loading: boolean
  error: Error | undefined
  dateRange: DateRange
  setDateRange: (dateRange: DateRange) => void
}) => {
  const [currency, setCurrency] = useState<Currency>("usd")
  const [layer, setLayer] = useState<Layers>("all")

  const net = data?.net
  const categories = data?.categories

  if (error) return <div className="text-destructive">{error.message}</div>
  if (loading && !data) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Profit and Loss Statement</CardTitle>
          <CardDescription>
            A financial report showing revenue, expenses, and resulting profit or loss
            over a specific time period.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <LoadingSkeleton />
        </CardContent>
      </Card>
    )
  }
  if (!net) return <div>No data</div>

  return (
    <Card>
      <CardHeader>
        <CardTitle>Profit and Loss Statement</CardTitle>
        <CardDescription>
          A financial report showing revenue, expenses, and resulting profit or loss over
          a specific time period.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div className="flex gap-6 items-center">
          <div>Date Range:</div>
          <DateRangeSelector initialDateRange={dateRange} onDateChange={setDateRange} />
        </div>

        <CurrencyLayerSelection
          currency={currency}
          setCurrency={setCurrency}
          layer={layer}
          setLayer={setLayer}
        />

        <Table className="mt-6">
          <TableBody>
            {categories?.map((category) => {
              return (
                <CategoryRow
                  key={category.name}
                  category={category}
                  currency={currency}
                  layer={layer}
                  transactionType={
                    BALANCE_FOR_CATEGORY[category.name].TransactionType || "netCredit"
                  }
                />
              )
            })}
          </TableBody>
          <TableFooter>
            <TableRow>
              <TableCell className="uppercase font-bold">NET</TableCell>
              <TableCell className="w-48">
                <Balance
                  align="end"
                  currency={currency}
                  amount={net[currency].closingBalance[layer].netCredit}
                />
              </TableCell>
            </TableRow>
          </TableFooter>
        </Table>
      </CardContent>
    </Card>
  )
}

const CategoryRow = ({
  category,
  currency,
  layer,
  transactionType,
}: {
  category: StatementCategory
  currency: Currency
  layer: Layers
  transactionType: TransactionType
}) => {
  return (
    <>
      <TableRow>
        <TableCell
          data-testid={`category-${category.name.toLowerCase()}`}
          className="flex items-center gap-2 text-primary font-semibold uppercase"
        >
          {category.name}
        </TableCell>
        <TableCell className="w-48">
          <Balance
            align="end"
            currency={currency}
            amount={category.amounts[currency].closingBalance[layer][transactionType]}
          />
        </TableCell>
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
    </>
  )
}
