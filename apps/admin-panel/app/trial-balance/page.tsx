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
import { Select } from "@/components/primitive/select"

gql`
  query GetOnBalanceSheetTrialBalance {
    trialBalance {
      name
      balance {
        ...balancesByCurrency
      }
      subAccounts {
        ... on AccountWithBalance {
          name
          balance {
            ...balancesByCurrency
          }
        }
        ... on AccountSetWithBalance {
          name
          balance {
            ...balancesByCurrency
          }
        }
      }
    }
  }

  query GetOffBalanceSheetTrialBalance {
    offBalanceSheetTrialBalance {
      name
      balance {
        ...balancesByCurrency
      }
      subAccounts {
        ... on AccountWithBalance {
          name
          balance {
            ...balancesByCurrency
          }
        }
        ... on AccountSetWithBalance {
          name
          balance {
            ...balancesByCurrency
          }
        }
      }
    }
  }

  fragment balancesByCurrency on AccountBalancesByCurrency {
    btc: btc {
      ...btcBalances
    }
    usd: usd {
      ...usdBalances
    }
  }

  fragment btcBalances on LayeredBtcAccountBalances {
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

  fragment usdBalances on LayeredUsdAccountBalances {
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
  currency: Currency
  layer: Layers
}
const TrialBalanceValues: React.FC<TrialBalanceValuesProps> = ({
  data,
  loading,
  error,
  currency,
  layer,
}) => {
  const balance = data?.balance
  const subAccounts = data?.subAccounts

  if (error) return <div className="text-destructive">{error.message}</div>
  if (loading) return <div>Loading...</div>
  if (!balance) return <div>No data</div>

  return (
    <>
      <Table>
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
                  currency={currency}
                  amount={memberBalance.balance[currency][layer].debit}
                />
              </TableCell>
              <TableCell className="w-48">
                <Balance
                  currency={currency}
                  amount={memberBalance.balance[currency][layer].credit}
                />
              </TableCell>
              <TableCell className="w-48">
                <Balance
                  currency={currency}
                  amount={memberBalance.balance[currency][layer].netDebit}
                />
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
        <TableFooter className="border-t-4">
          <TableRow>
            <TableCell className="text-right uppercase font-bold pr-10">Totals</TableCell>
            <TableCell className="w-48">
              <Balance currency={currency} amount={balance[currency][layer].debit} />
            </TableCell>
            <TableCell className="w-48">
              <Balance currency={currency} amount={balance[currency][layer].credit} />
            </TableCell>
            <TableCell className="w-48">
              <Balance currency={currency} amount={balance[currency][layer].netDebit} />
            </TableCell>
          </TableRow>
        </TableFooter>
      </Table>
    </>
  )
}

function TrialBalancePage() {
  const [currency, setCurrency] = React.useState<Currency>("usd")
  const [layer, setLayer] = React.useState<Layers>("all")
  const {
    data: onBalanceSheetData,
    loading: onBalanceSheetLoading,
    error: onBalanceSheetError,
  } = useGetOnBalanceSheetTrialBalanceQuery()
  const {
    data: offBalanceSheetData,
    loading: offBalanceSheetLoading,
    error: offBalanceSheetError,
  } = useGetOffBalanceSheetTrialBalanceQuery()

  return (
    <main>
      <Tabs defaultValue="onBalanceSheet">
        <div className="flex justify-between">
          <PageHeading>Trial Balance</PageHeading>
          <div className="flex gap-4">
            <TabsList>
              <TabsTrigger value="onBalanceSheet">Regular</TabsTrigger>
              <TabsTrigger value="offBalanceSheet">Off Balance Sheet</TabsTrigger>
            </TabsList>
            <div className="w-32">
              <Select
                value={currency}
                onChange={(e) => setCurrency(e.target.value as Currency)}
              >
                <option value="btc">BTC</option>
                <option value="usd">USD</option>
              </Select>
            </div>
            <div className="w-32">
              <Select value={layer} onChange={(e) => setLayer(e.target.value as Layers)}>
                <option value="all">All</option>
                <option value="settled">Settled</option>
                <option value="pending">Pending</option>
                <option value="encumbrance">Encumbrance</option>
              </Select>
            </div>
          </div>
        </div>

        <TabsContent value="onBalanceSheet">
          <TrialBalanceValues
            data={onBalanceSheetData?.trialBalance}
            loading={onBalanceSheetLoading}
            error={onBalanceSheetError}
            currency={currency}
            layer={layer}
          />
        </TabsContent>
        <TabsContent value="offBalanceSheet">
          <TrialBalanceValues
            currency={currency}
            layer={layer}
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
