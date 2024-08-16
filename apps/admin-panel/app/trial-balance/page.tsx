"use client"
import React from "react"
import { ApolloError, gql } from "@apollo/client"

import { PageHeading } from "@/components/page-heading"
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
} from "@/components/primitive/table"
import { Tabs, TabsList, TabsContent, TabsTrigger } from "@/components/primitive/tab"

import Balance, { Currency } from "@/components/balance/balance"
import { CurrencyLayerSelection } from "@/components/financial/currency-layer-selection"

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

type Layers = "all" | "settled" | "pending" | "encumbrance"
type TrialBalanceValuesProps = {
  data:
    | GetOffBalanceSheetTrialBalanceQuery["offBalanceSheetTrialBalance"]
    | GetOnBalanceSheetTrialBalanceQuery["trialBalance"]
    | undefined
  loading: boolean
  error: ApolloError | undefined
}
const TrialBalanceValues: React.FC<TrialBalanceValuesProps> = ({
  data,
  loading,
  error,
}) => {
  const [currency, setCurrency] = React.useState<Currency>("usd")
  const [layer, setLayer] = React.useState<Layers>("all")

  const total = data?.total
  const subAccounts = data?.subAccounts

  if (error) return <div className="text-destructive">{error.message}</div>
  if (loading) return <div>Loading...</div>
  if (!total) return <div>No data</div>

  return (
    <>
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
            <TableCell className="text-right uppercase font-bold pr-10">Totals</TableCell>
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
                amount={total[currency].closingBalance[layer].credit}
              />
            </TableCell>
            <TableCell className="w-48">
              <Balance
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
  const {
    data: onBalanceSheetData,
    loading: onBalanceSheetLoading,
    error: onBalanceSheetError,
  } = useGetOnBalanceSheetTrialBalanceQuery({
    variables: {
      from: new Date(Date.now()),
    },
  })
  const {
    data: offBalanceSheetData,
    loading: offBalanceSheetLoading,
    error: offBalanceSheetError,
  } = useGetOffBalanceSheetTrialBalanceQuery({
    variables: {
      from: new Date(Date.now()),
    },
  })

  return (
    <main>
      <PageHeading>Trial Balance</PageHeading>
      <Tabs defaultValue="onBalanceSheet">
        <TabsList className="mb-4">
          <TabsTrigger value="onBalanceSheet">Regular</TabsTrigger>
          <TabsTrigger value="offBalanceSheet">Off Balance Sheet</TabsTrigger>
        </TabsList>
        <TabsContent value="onBalanceSheet">
          <TrialBalanceValues
            data={onBalanceSheetData?.trialBalance}
            loading={onBalanceSheetLoading}
            error={onBalanceSheetError}
          />
        </TabsContent>
        <TabsContent value="offBalanceSheet">
          <TrialBalanceValues
            data={offBalanceSheetData?.offBalanceSheetTrialBalance}
            loading={offBalanceSheetLoading}
            error={offBalanceSheetError}
          />
        </TabsContent>
      </Tabs>
    </main>
  )
}

export default TrialBalancePage
