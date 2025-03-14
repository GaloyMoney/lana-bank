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
  CashFlowStatementQuery,
  useCashFlowStatementQuery,
} from "@/lib/graphql/generated"
import Balance, { Currency } from "@/components/balance/balance"
import { CurrencyLayerSelection } from "@/components/financial/currency-layer-selection"
import {
  DateRange,
  DateRangeSelector,
  getInitialDateRange,
} from "@/components/date-range-picker"

const BALANCE_FOR_CATEGORY: {
  [key: string]: { TransactionType: TransactionType }
} = {
  "Cash Flow From Operations": { TransactionType: "netCredit" },
  "Cash Flow From Investing": { TransactionType: "netCredit" },
  "Cash Flow From Financing": { TransactionType: "netCredit" },
}

gql`
  fragment basicBtcAmounts on BtcAccountBalanceAmounts {
    debit
    credit
    netDebit
    netCredit
  }

  fragment basicUsdAmounts on UsdAccountBalanceAmounts {
    debit
    credit
    netDebit
    netCredit
  }

  fragment layeredBtcAmounts on LayeredBtcAccountBalanceAmounts {
    all {
      ...basicBtcAmounts
    }
    settled {
      ...basicBtcAmounts
    }
    pending {
      ...basicBtcAmounts
    }
    encumbrance {
      ...basicBtcAmounts
    }
  }

  fragment layeredUsdAmounts on LayeredUsdAccountBalanceAmounts {
    all {
      ...basicUsdAmounts
    }
    settled {
      ...basicUsdAmounts
    }
    pending {
      ...basicUsdAmounts
    }
    encumbrance {
      ...basicUsdAmounts
    }
  }

  fragment btcAmountsInPeriod on BtcAccountAmountsInPeriod {
    openingBalance {
      ...layeredBtcAmounts
    }
    closingBalance {
      ...layeredBtcAmounts
    }
    amount {
      ...layeredBtcAmounts
    }
  }

  fragment usdAmountsInPeriod on UsdAccountAmountsInPeriod {
    openingBalance {
      ...layeredUsdAmounts
    }
    closingBalance {
      ...layeredUsdAmounts
    }
    amount {
      ...layeredUsdAmounts
    }
  }

  fragment amountsByCurrency on AccountAmountsByCurrency {
    btc {
      ...btcAmountsInPeriod
    }
    usd {
      ...usdAmountsInPeriod
    }
  }

  query CashFlowStatement($from: Timestamp!, $until: Timestamp) {
    cashFlowStatement(from: $from, until: $until) {
      name
      total {
        ...amountsByCurrency
      }
      categories {
        name
        accounts {
          ... on Account {
            __typename
            id
            name
            amounts {
              ...amountsByCurrency
            }
          }
          ... on AccountSet {
            __typename
            id
            name
            amounts {
              ...amountsByCurrency
            }
          }
        }
        amounts {
          ...amountsByCurrency
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
          {[1, 2, 3].map((i) => (
            <TableRow key={`skeleton-row-${i}`}>
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
        </TableBody>
      </Table>
    </div>
  )
}
const CategoryRow = ({
  category,
  currency,
  layer,
  transactionType,
}: {
  category: NonNullable<CashFlowStatementQuery["cashFlowStatement"]>["categories"][number]
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

export default function CashFlowStatementPage() {
  const [dateRange, setDateRange] = useState<DateRange>(getInitialDateRange())
  const handleDateChange = useCallback((newDateRange: DateRange) => {
    setDateRange(newDateRange)
  }, [])

  const {
    data: cashFlowStatementData,
    loading: cashFlowStatementLoading,
    error: cashFlowStatementError,
  } = useCashFlowStatementQuery({
    variables: dateRange,
  })

  return (
    <CashFlowStatement
      data={cashFlowStatementData?.cashFlowStatement}
      loading={cashFlowStatementLoading && !cashFlowStatementData}
      error={cashFlowStatementError}
      dateRange={dateRange}
      setDateRange={handleDateChange}
    />
  )
}

const CashFlowStatement = ({
  data,
  loading,
  error,
  dateRange,
  setDateRange,
}: {
  data?: CashFlowStatementQuery["cashFlowStatement"]
  loading: boolean
  error: Error | undefined
  dateRange: DateRange
  setDateRange: (dateRange: DateRange) => void
}) => {
  const t = useTranslations("CashFlowStatement")
  const [currency, setCurrency] = useState<Currency>("usd")
  const [layer, setLayer] = useState<Layers>("all")

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

  if (!data?.total) return <div>{t("noData")}</div>

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

        <CurrencyLayerSelection
          currency={currency}
          setCurrency={setCurrency}
          layer={layer}
          setLayer={setLayer}
        />

        <Table className="mt-6">
          <TableBody>
            {data.categories?.map((category) => (
              <CategoryRow
                key={category.name}
                category={category}
                currency={currency}
                layer={layer}
                transactionType={
                  BALANCE_FOR_CATEGORY[category.name]?.TransactionType || "netCredit"
                }
              />
            ))}
          </TableBody>
          <TableFooter>
            <TableRow>
              <TableCell className="uppercase font-bold">{t("total")}</TableCell>
              <TableCell className="w-48">
                <Balance
                  align="end"
                  currency={currency}
                  amount={data.total[currency].closingBalance[layer].netCredit}
                />
              </TableCell>
            </TableRow>
          </TableFooter>
        </Table>
      </CardContent>
    </Card>
  )
}
