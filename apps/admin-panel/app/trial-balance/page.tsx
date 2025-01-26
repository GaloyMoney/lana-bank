"use client"
import React, { useCallback, useState } from "react"
import { ApolloError, gql } from "@apollo/client"

import {
  GetOffBalanceSheetTrialBalanceQuery,
  GetOnBalanceSheetTrialBalanceQuery,
  useGetOffBalanceSheetTrialBalanceQuery,
  useGetOnBalanceSheetTrialBalanceQuery,
} from "@/lib/graphql/generated"

import {
  Table,
  TableBody,
  TableCell,
  TableFooter,
  TableHead,
  TableHeader,
  TableRow,
} from "@/ui/table"
import { Tabs, TabsList, TabsContent, TabsTrigger } from "@/ui/tab"

import Balance, { Currency } from "@/components/balance/balance"
import { CurrencyLayerSelection } from "@/components/financial/currency-layer-selection"
import {
  DateRange,
  DateRangeSelector,
  getInitialDateRange,
} from "@/components/date-range-picker"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/ui/card"
import { Skeleton } from "@/ui/skeleton"

gql`
  query GetOnBalanceSheetTrialBalance($from: Timestamp!, $until: Timestamp) {
    trialBalance(from: $from, until: $until) {
      name
      total {
        ...balancesByCurrency
      }
      subAccounts {
        ... on Account {
          name
          amounts {
            ...balancesByCurrency
          }
        }
        ... on AccountSet {
          name
          amounts {
            ...balancesByCurrency
          }
        }
      }
    }
  }

  query GetOffBalanceSheetTrialBalance($from: Timestamp!, $until: Timestamp) {
    offBalanceSheetTrialBalance(from: $from, until: $until) {
      name
      total {
        ...balancesByCurrency
      }
      subAccounts {
        ... on Account {
          name
          amounts {
            ...balancesByCurrency
          }
        }
        ... on AccountSet {
          name
          amounts {
            ...balancesByCurrency
          }
        }
      }
    }
  }

  fragment balancesByCurrency on AccountAmountsByCurrency {
    btc: btc {
      ...rangedBtcBalances
    }
    usd: usd {
      ...rangedUsdBalances
    }
  }

  fragment rangedBtcBalances on BtcAccountAmountsInPeriod {
    openingBalance {
      ...btcBalances
    }
    closingBalance {
      ...btcBalances
    }
    amount {
      ...btcBalances
    }
  }

  fragment rangedUsdBalances on UsdAccountAmountsInPeriod {
    openingBalance {
      ...usdBalances
    }
    closingBalance {
      ...usdBalances
    }
    amount {
      ...usdBalances
    }
  }

  fragment btcBalances on LayeredBtcAccountAmounts {
    all {
      debit
      credit
      netDebit
      netCredit
    }
    settled {
      debit
      credit
      netDebit
      netCredit
    }
    pending {
      debit
      credit
      netDebit
      netCredit
    }
    encumbrance {
      debit
      credit
      netDebit
      netCredit
    }
  }

  fragment usdBalances on LayeredUsdAccountAmounts {
    all {
      debit
      credit
      netDebit
      netCredit
    }
    settled {
      debit
      credit
      netDebit
      netCredit
    }
    pending {
      debit
      credit
      netDebit
      netCredit
    }
    encumbrance {
      debit
      credit
      netDebit
      netCredit
    }
  }
`

const LoadingSkeleton = () => {
  return (
    <div className="space-y-4" data-testid="loading-skeleton">
      <div className="space-y-4">
        <Skeleton className="h-10 w-72" />
        <Skeleton className="h-10 w-96" />
      </div>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>
              <Skeleton className="h-5 w-32" />
            </TableHead>
            <TableHead>
              <Skeleton className="h-5 w-24 ml-auto" />
            </TableHead>
            <TableHead>
              <Skeleton className="h-5 w-24 ml-auto" />
            </TableHead>
            <TableHead>
              <Skeleton className="h-5 w-24 ml-auto" />
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {[1, 2, 3, 4, 5].map((i) => (
            <TableRow key={i}>
              <TableCell>
                <Skeleton className="h-5 w-full" />
              </TableCell>
              <TableCell>
                <Skeleton className="h-5 w-24 ml-auto" />
              </TableCell>
              <TableCell>
                <Skeleton className="h-5 w-24 ml-auto" />
              </TableCell>
              <TableCell>
                <Skeleton className="h-5 w-24 ml-auto" />
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  )
}

type Layers = "all" | "settled" | "pending"
type TrialBalanceValuesProps = {
  data:
    | GetOffBalanceSheetTrialBalanceQuery["offBalanceSheetTrialBalance"]
    | GetOnBalanceSheetTrialBalanceQuery["trialBalance"]
    | undefined
  loading: boolean
  error: ApolloError | undefined
  dateRange: DateRange
  setDateRange: (dateRange: DateRange) => void
}
const TrialBalanceValues: React.FC<TrialBalanceValuesProps> = ({
  data,
  loading,
  error,
  dateRange,
  setDateRange,
}) => {
  const [currency, setCurrency] = React.useState<Currency>("usd")
  const [layer, setLayer] = React.useState<Layers>("all")

  const total = data?.total
  const subAccounts = data?.subAccounts

  if (error) return <div className="text-destructive">{error.message}</div>
  if (loading && !data) {
    return <LoadingSkeleton />
  }
  if (!total) return <div>No data</div>

  return (
    <>
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
        <TableHeader>
          <TableHead>Account Name</TableHead>
          <TableHead className="text-right">Debit</TableHead>
          <TableHead className="text-right">Credit</TableHead>
          <TableHead className="text-right">Net</TableHead>
        </TableHeader>
        <TableBody>
          {subAccounts?.map((memberBalance, index) => (
            <TableRow key={index}>
              <TableCell>{memberBalance.name}</TableCell>
              <TableCell className="w-48">
                <Balance
                  align="end"
                  currency={currency}
                  amount={memberBalance.amounts[currency].closingBalance[layer].debit}
                />
              </TableCell>
              <TableCell className="w-48">
                <Balance
                  align="end"
                  currency={currency}
                  amount={memberBalance.amounts[currency].closingBalance[layer].credit}
                />
              </TableCell>
              <TableCell className="w-48">
                <Balance
                  align="end"
                  currency={currency}
                  amount={memberBalance.amounts[currency].closingBalance[layer].netDebit}
                />
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
        <TableFooter className="border-t-4">
          <TableRow>
            <TableCell className="uppercase font-bold">Totals</TableCell>
            <TableCell className="w-48">
              <Balance
                align="end"
                currency={currency}
                amount={total[currency].closingBalance[layer].debit}
              />
            </TableCell>
            <TableCell className="w-48">
              <Balance
                align="end"
                currency={currency}
                amount={total[currency].closingBalance[layer].credit}
              />
            </TableCell>
            <TableCell className="w-48">
              <Balance
                align="end"
                currency={currency}
                amount={total[currency].closingBalance[layer].netDebit}
              />
            </TableCell>
          </TableRow>
        </TableFooter>
      </Table>
    </>
  )
}
function TrialBalancePage() {
  const [dateRange, setDateRange] = useState<DateRange>(getInitialDateRange)
  const handleDateChange = useCallback((newDateRange: DateRange) => {
    setDateRange(newDateRange)
  }, [])

  const {
    data: onBalanceSheetData,
    loading: onBalanceSheetLoading,
    error: onBalanceSheetError,
  } = useGetOnBalanceSheetTrialBalanceQuery({
    variables: {
      from: dateRange.from,
      until: dateRange.until,
    },
  })
  const {
    data: offBalanceSheetData,
    loading: offBalanceSheetLoading,
    error: offBalanceSheetError,
  } = useGetOffBalanceSheetTrialBalanceQuery({
    variables: {
      from: dateRange.from,
      until: dateRange.until,
    },
  })

  return (
    <Card>
      <CardHeader>
        <CardTitle>Trial Balance</CardTitle>
        <CardDescription>
          A summary of all accounts listing debit and credit balances to ensure accounting
          entries are balanced.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <Tabs defaultValue="onBalanceSheet">
          <TabsList className="mb-4">
            <TabsTrigger value="onBalanceSheet">Regular</TabsTrigger>
            <TabsTrigger value="offBalanceSheet">Off Balance Sheet</TabsTrigger>
          </TabsList>
          <TabsContent value="onBalanceSheet">
            <TrialBalanceValues
              data={onBalanceSheetData?.trialBalance}
              loading={onBalanceSheetLoading && !onBalanceSheetData}
              error={onBalanceSheetError}
              dateRange={dateRange}
              setDateRange={handleDateChange}
            />
          </TabsContent>
          <TabsContent value="offBalanceSheet">
            <TrialBalanceValues
              data={offBalanceSheetData?.offBalanceSheetTrialBalance}
              loading={offBalanceSheetLoading && !offBalanceSheetData}
              error={offBalanceSheetError}
              dateRange={dateRange}
              setDateRange={handleDateChange}
            />
          </TabsContent>
        </Tabs>
      </CardContent>
    </Card>
  )
}

export default TrialBalancePage
