"use client"
import React from "react"

import { PageHeading } from "@/components/page-heading"
import { useGetTrialBalanceQuery } from "@/lib/graphql/generated"
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

import Balance, { Currency } from "@/components/balance/balance"

type Layers = "all" | "settled" | "pending" | "encumbrance"

function TrialBalancePage() {
  const [currency, setCurrency] = React.useState<Currency>("btc")
  const [layer, setLayer] = React.useState<Layers>("all")

  const { data, loading } = useGetTrialBalanceQuery()

  const balance = data?.trialBalance?.balance
  const subAccounts = data?.trialBalance?.subAccounts

  if (loading || !balance) return <div>Loading...</div>

  return (
    <main>
      <PageHeading>Trial Balance</PageHeading>
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
    </main>
  )
}

export default TrialBalancePage
