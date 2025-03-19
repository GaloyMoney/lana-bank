"use client"
import React, { useCallback, useState } from "react"
import { ApolloError, gql } from "@apollo/client"

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

import { Skeleton } from "@lana/web/ui/skeleton"

import { useRouter } from "next/navigation"
import { useTranslations } from "next-intl"

import { GetTrialBalanceQuery, useGetTrialBalanceQuery } from "@/lib/graphql/generated"

import Balance, { Currency } from "@/components/balance/balance"
import { CurrencyLayerSelection } from "@/components/financial/currency-layer-selection"
import {
  DateRange,
  DateRangeSelector,
  getInitialDateRange,
} from "@/components/date-range-picker"

gql`
  query GetTrialBalance(
    $from: Timestamp!
    $until: Timestamp!
    $first: Int!
    $after: String
  ) {
    trialBalance(from: $from, until: $until) {
      name
      total {
        ...balancesByCurrency
      }
      accounts(first: $first, after: $after) {
        edges {
          cursor
          node {
            id
            code
            name
            amounts {
              ...balancesByCurrency
            }
          }
        }
        pageInfo {
          endCursor
          startCursor
          hasNextPage
          hasPreviousPage
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

  fragment btcBalances on LayeredBtcAccountBalanceAmounts {
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

  fragment usdBalances on LayeredUsdAccountBalanceAmounts {
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
  data: GetTrialBalanceQuery["trialBalance"] | undefined
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
  const t = useTranslations("TrialBalance")
  const [currency, setCurrency] = React.useState<Currency>("usd")
  const [layer, setLayer] = React.useState<Layers>("all")

  const router = useRouter()

  const total = data?.total
  const accounts = data?.accounts

  if (error) return <div className="text-destructive">{error.message}</div>
  if (loading && !data) {
    return <LoadingSkeleton />
  }
  if (!total) return <div>{t("noData")}</div>

  return (
    <>
      <div className="flex gap-6 items-center">
        <div>{t("dateRange")}:</div>
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
          <TableHead>{t("table.headers.accountName")}</TableHead>
          <TableHead>{t("table.headers.accountCode")}</TableHead>
          <TableHead className="text-right">{t("table.headers.debit")}</TableHead>
          <TableHead className="text-right">{t("table.headers.credit")}</TableHead>
          <TableHead className="text-right">{t("table.headers.net")}</TableHead>
        </TableHeader>
        <TableBody>
          {accounts?.map((account, index) => (
            <TableRow
              className="cursor-pointer"
              onClick={() => router.push(`/ledger-account/${account.code}`)}
              key={index}
            >
              <TableCell>{account.name}</TableCell>
              <TableCell className="w-48">
                <div className="font-mono text-xs text-gray-500">{account.code}</div>
              </TableCell>
              <TableCell className="w-48">
                <Balance
                  align="end"
                  currency={currency}
                  amount={account.amounts[currency].closingBalance[layer].debit}
                />
              </TableCell>
              <TableCell className="w-48">
                <Balance
                  align="end"
                  currency={currency}
                  amount={account.amounts[currency].closingBalance[layer].credit}
                />
              </TableCell>
              <TableCell className="w-48">
                <Balance
                  align="end"
                  currency={currency}
                  amount={account.amounts[currency].closingBalance[layer].netDebit}
                />
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
        <TableFooter className="border-t-4">
          <TableRow>
            <TableCell className="uppercase font-bold">{t("totals")}</TableCell>
            <TableCell></TableCell>
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
  const t = useTranslations("TrialBalance")
  const [dateRange, setDateRange] = useState<DateRange>(getInitialDateRange)
  const handleDateChange = useCallback((newDateRange: DateRange) => {
    setDateRange(newDateRange)
  }, [])

  const { data, loading, error } = useGetTrialBalanceQuery({
    variables: {
      from: dateRange.from,
      until: dateRange.until,
    },
  })

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("title")}</CardTitle>
        <CardDescription>{t("description")}</CardDescription>
      </CardHeader>
      <CardContent>
        <TrialBalanceValues
          data={data?.trialBalance}
          loading={loading && !data}
          error={error}
          dateRange={dateRange}
          setDateRange={handleDateChange}
        />
      </CardContent>
    </Card>
  )
}

export default TrialBalancePage
