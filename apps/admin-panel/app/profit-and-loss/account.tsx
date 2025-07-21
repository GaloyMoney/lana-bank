"use client"
import { TableCell, TableRow } from "@lana/web/ui/table"

import Link from "next/link"

import Balance, { Currency } from "@/components/balance/balance"
import { ProfitAndLossStatementQuery } from "@/lib/graphql/generated"

type AccountType = NonNullable<
  ProfitAndLossStatementQuery["profitAndLossStatement"]
>["categories"][0]["children"][number]

interface AccountProps {
  account: AccountType
  currency: Currency
  depth?: number
  layer: PnlLayers
}

export const Account = ({ account, currency, depth = 0, layer }: AccountProps) => {
  let accountEnd: number | undefined

  if (account.balanceRange.__typename === "UsdLedgerAccountBalanceRange") {
    accountEnd = account.balanceRange.usdEnd[layer].net
  } else if (account.balanceRange.__typename === "BtcLedgerAccountBalanceRange") {
    accountEnd = account.balanceRange.btcEnd[layer].net
  }

  return (
    <TableRow data-testid={`account-${account.id}`}>
      <Link href={`/ledger-accounts/${account.code || account.id}`}>
        <TableCell className="flex items-center">
          {Array.from({ length: depth }).map((_, i) => (
            <div key={i} className="w-8" />
          ))}
          <div className="w-8" />
          <div>{account.name}</div>
        </TableCell>
      </Link>
      <TableCell>
        <Balance align="end" currency={currency} amount={accountEnd as CurrencyType} />
      </TableCell>
    </TableRow>
  )
}
