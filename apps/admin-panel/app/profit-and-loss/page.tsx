"use client"
import { gql } from "@apollo/client"
import { useCallback, useState } from "react"

import { Account } from "./account"

import {
  ProfitAndLossStatementQuery,
  StatementCategory,
  useProfitAndLossStatementQuery,
} from "@/lib/graphql/generated"
import Balance, { Currency } from "@/components/balance/balance"
import {
  Table,
  TableBody,
  TableCell,
  TableFooter,
  TableRow,
} from "@/components/primitive/table"

import { PageHeading } from "@/components/page-heading"
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
            hasSubAccounts
            amounts {
              ...balancesByCurrency
            }
          }
        }
      }
    }
  }
`

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
      loading={ProfitAndLossStatementLoading}
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
  data: ProfitAndLossStatementQuery["profitAndLossStatement"]
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
  if (loading) return <div>Loading...</div>
  if (!net) return <div>No data</div>

  return (
    <main>
      <div>
        <PageHeading>Profit and Loss</PageHeading>
        <div className="mt-6 flex gap-6 items-center">
          <div>Date Range:</div>
          <DateRangeSelector initialDateRange={dateRange} onDateChange={setDateRange} />
        </div>
        <CurrencyLayerSelection
          currency={currency}
          setCurrency={setCurrency}
          layer={layer}
          setLayer={setLayer}
        />
      </div>
      <Table className="mt-6">
        <TableBody>
          {categories?.map((category) => {
            return (
              <CategoryRow
                key={category.name}
                category={category}
                currency={currency}
                layer={layer}
                dateRange={dateRange}
                transactionType={
                  BALANCE_FOR_CATEGORY[category.name].TransactionType || "netCredit"
                }
              />
            )
          })}
        </TableBody>
        <TableFooter>
          <TableRow>
            <TableCell className="uppercase pr-10 text-textColor-secondary">
              NET
            </TableCell>
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
    </main>
  )
}

const CategoryRow = ({
  category,
  currency,
  layer,
  transactionType,
  dateRange,
}: {
  category: StatementCategory
  currency: Currency
  layer: Layers
  transactionType: TransactionType
  dateRange: DateRange
}) => {
  console.log(category.name, transactionType)

  return (
    <>
      <TableRow>
        <TableCell className="flex items-center gap-2 text-primary font-semibold uppercase">
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
          dateRange={dateRange}
        />
      ))}
    </>
  )
}
