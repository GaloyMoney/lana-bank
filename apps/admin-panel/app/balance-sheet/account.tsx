"use client"
import { TableCell, TableRow } from "@lana/web/ui/table"

import { useRouter } from "next/navigation"

import Balance, { Currency } from "@/components/balance/balance"
import { BalanceSheetQuery } from "@/lib/graphql/generated"

interface AccountProps {
  account: NonNullable<BalanceSheetQuery["balanceSheet"]>["categories"][0]["children"][0]
  currency: Currency
  depth?: number
  layer: BalanceSheetLayers
}

export const Account = ({ account, currency, depth = 0, layer }: AccountProps) => {
  const router = useRouter()

  let balance: number = 0

  if (account.balanceRange) {
    if (
      account.balanceRange.__typename === "UsdLedgerAccountBalanceRange" &&
      currency === "usd"
    ) {
      balance = account.balanceRange.usdEnd[layer].net
    } else if (
      account.balanceRange.__typename === "BtcLedgerAccountBalanceRange" &&
      currency === "btc"
    ) {
      balance = account.balanceRange.btcEnd[layer].net
    }
  }

  const handleRowClick = () => {
    router.push(`/ledger-accounts/${account.code || account.id}`)
  }

  return (
    <TableRow
      key={account.id}
      className="cursor-pointer hover:bg-muted/50"
      onClick={handleRowClick}
    >
      <TableCell className="flex items-center">
        {Array.from({ length: depth }).map((_, i) => (
          <div key={i} className="w-8" />
        ))}
        <div className="w-8" />
        <div>{account.name}</div>
      </TableCell>
      <TableCell>
        <Balance
          align="end"
          className="font-semibold"
          currency={currency}
          amount={balance as CurrencyType}
        />
      </TableCell>
    </TableRow>
  )
}
