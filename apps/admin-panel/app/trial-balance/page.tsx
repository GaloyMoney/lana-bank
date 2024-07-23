"use client"
import React from "react"
import { gql } from "@apollo/client"

import { PageHeading } from "@/components/page-heading"
import {
  GetOffBalanceSheetTrialBalanceQuery,
  GetOnBalanceSheetTrialBalanceQuery,
  useGetOffBalanceSheetTrialBalanceQuery,
  useGetOnBalanceSheetTrialBalanceQuery,
} from "@/lib/graphql/generated"
import { RadioGroup, RadioGroupItem } from "@/components/primitive/radio-group"
import { Label } from "@/components/primitive/label"
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
    usdt: usdt {
      ...usdBalances
    }
  }

  fragment btcBalances on LayeredBtcAccountBalances {
    all {
      netDebit
      debit
      credit
    }
    settled {
      netDebit
      debit
      credit
    }
    pending {
      netDebit
      debit
      credit
    }
    encumbrance {
      netDebit
      debit
      credit
    }
  }

  fragment usdBalances on LayeredUsdAccountBalances {
    all {
      netDebit
      debit
      credit
    }
    settled {
      netDebit
      debit
      credit
    }
    pending {
      netDebit
      debit
      credit
    }
    encumbrance {
      netDebit
      debit
      credit
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
}
const TrialBalanceValues: React.FC<TrialBalanceValuesProps> = ({ data, loading }) => {
  const [currency, setCurrency] = React.useState<Currency>("btc")
  const [layer, setLayer] = React.useState<Layers>("all")

  const balance = data?.balance
  const subAccounts = data?.subAccounts

  if (loading || !balance) return <div>Loading...</div>

  return (
    <>
      <div>
        <div className="flex items-center mt-2">
          <div className="w-28">Currency:</div>
          <RadioGroup
            className="flex items-center space-x-4"
            defaultValue={"btc"}
            value={currency}
            onValueChange={(v: Currency) => setCurrency(v)}
          >
            <div className="flex items-center space-x-2">
              <RadioGroupItem value="btc" id="currency-btc" />
              <Label htmlFor="currency-btc">BTC</Label>
            </div>
            <div className="flex items-center space-x-2">
              <RadioGroupItem value="usd" id="currency-usd" />
              <Label htmlFor="currency-usd">USD</Label>
            </div>
            <div className="flex items-center space-x-2">
              <RadioGroupItem value="usdt" id="currency-usdt" />
              <Label htmlFor="currency-usdt">USDT</Label>
            </div>
          </RadioGroup>
        </div>
        <div className="flex items-center mt-2">
          <div className="w-28">Layer:</div>
          <RadioGroup
            className="flex items-center space-x-4"
            defaultValue={"all"}
            value={layer}
            onValueChange={(v: Layers) => setLayer(v)}
          >
            <div className="flex items-center space-x-2">
              <RadioGroupItem value="all" id="layer-all" />
              <Label htmlFor="layer-all">All</Label>
            </div>
            <div className="flex items-center space-x-2">
              <RadioGroupItem value="settled" id="layer-settled" />
              <Label htmlFor="layer-settled">Settled</Label>
            </div>
            <div className="flex items-center space-x-2">
              <RadioGroupItem value="pending" id="layer-pending" />
              <Label htmlFor="layer-pending">Pending</Label>
            </div>
            <div className="flex items-center space-x-2">
              <RadioGroupItem value="encumbrance" id="layer-encumbrance" />
              <Label htmlFor="layer-encumbrance">Encumbrance</Label>
            </div>
          </RadioGroup>
        </div>
      </div>

      <Table className="mt-4">
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
  const { data: onBalanceSheetData, loading: onBalanceSheetLoading } =
    useGetOnBalanceSheetTrialBalanceQuery()
  const { data: offBalanceSheetData, loading: offBalanceSheetLoading } =
    useGetOffBalanceSheetTrialBalanceQuery()

  return (
    <main>
      <PageHeading>Trial Balance</PageHeading>
      <Tabs defaultValue="onBalanceSheet">
        <TabsList>
          <TabsTrigger value="onBalanceSheet">Regular</TabsTrigger>
          <TabsTrigger value="offBalanceSheet">Off Balance Sheet</TabsTrigger>
        </TabsList>
        <TabsContent value="onBalanceSheet">
          <TrialBalanceValues
            data={onBalanceSheetData?.trialBalance}
            loading={onBalanceSheetLoading}
          />
        </TabsContent>
        <TabsContent value="offBalanceSheet">
          <TrialBalanceValues
            data={offBalanceSheetData?.offBalanceSheetTrialBalance}
            loading={offBalanceSheetLoading}
          />
        </TabsContent>
      </Tabs>
    </main>
  )
}

export default TrialBalancePage
