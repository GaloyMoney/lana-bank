"use client"
import React from "react"

import { PageHeading } from "@/components/page-heading"
import { useGetTrialBalanceQuery } from "@/lib/graphql/generated"
import { Checkbox } from "@/components/primitive/check-box"
import { RadioGroup, RadioGroupItem } from "@/components/primitive/radio-group"
import { Label } from "@/components/primitive/label"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/primitive/table"

type Layers = "all" | "settled" | "pending" | "encumbrance"

function TrialBalancePage() {
  const [showBtc, setShowBtc] = React.useState(true)
  const [showUsd, setShowUsd] = React.useState(true)
  const [showUsdt, setShowUsdt] = React.useState(true)

  const [layer, setLayer] = React.useState<Layers>("all")

  const { data } = useGetTrialBalanceQuery()
  const balance = data?.trialBalance?.balance

  return (
    <main>
      <PageHeading>Trial Balance</PageHeading>
      <div>
        <div className="flex items-center">
          <div className="w-28">Currency:</div>
          <div className="flex items-center space-x-4">
            <div className="flex items-center space-x-2">
              <Checkbox
                id="btc"
                checked={showBtc}
                onCheckedChange={(v: boolean) => setShowBtc(v)}
              />
              <Label htmlFor="btc">BTC</Label>
            </div>
            <div className="flex items-center space-x-2">
              <Checkbox
                id="usd"
                checked={showUsd}
                onCheckedChange={(v: boolean) => setShowUsd(v)}
              />
              <Label htmlFor="usd">USD</Label>
            </div>
            <div className="flex items-center space-x-2">
              <Checkbox
                id="usdt"
                checked={showUsdt}
                onCheckedChange={(v: boolean) => setShowUsdt(v)}
              />
              <Label htmlFor="usdt">USDT</Label>
            </div>
          </div>
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
              <RadioGroupItem value="pending" id="layer-pending" />
              <Label htmlFor="layer-pending">Pending</Label>
            </div>
            <div className="flex items-center space-x-2">
              <RadioGroupItem value="settled" id="layer-settled" />
              <Label htmlFor="layer-settled">Settled</Label>
            </div>
            <div className="flex items-center space-x-2">
              <RadioGroupItem value="encumbrance" id="layer-encumbrance" />
              <Label htmlFor="layer-encumbrance">Encumbrance</Label>
            </div>
          </RadioGroup>
        </div>
      </div>
      <Table className="mt-4 max-w-2xl">
        <TableHeader>
          <TableHead>BALANCES</TableHead>
          <TableHead>Credit</TableHead>
          <TableHead>Debit</TableHead>
          <TableHead>Net</TableHead>
        </TableHeader>
        <TableBody>
          {showBtc && (
            <TableRow>
              <TableCell>BTC</TableCell>
              <TableCell className="max-w-10">{balance?.btc[layer].credit}</TableCell>
              <TableCell className="max-w-10">{balance?.btc[layer].debit}</TableCell>
              <TableCell className="max-w-10">{balance?.btc[layer].net}</TableCell>
            </TableRow>
          )}
          {showUsd && (
            <TableRow>
              <TableCell>USD</TableCell>
              <TableCell className="max-w-10">{balance?.usd[layer].credit}</TableCell>
              <TableCell className="max-w-10">{balance?.usd[layer].debit}</TableCell>
              <TableCell className="max-w-10">{balance?.usd[layer].net}</TableCell>
            </TableRow>
          )}
          {showUsdt && (
            <TableRow>
              <TableCell>USDT</TableCell>
              <TableCell className="max-w-10">{balance?.usdt[layer].credit}</TableCell>
              <TableCell className="max-w-10">{balance?.usdt[layer].debit}</TableCell>
              <TableCell className="max-w-10">{balance?.usdt[layer].net}</TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>
    </main>
  )
}

export default TrialBalancePage
