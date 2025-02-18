"use client"
import { TableCell, TableRow } from "@lana/web/ui/table"

import { AccountSetSubAccount } from "@/lib/graphql/generated"
import Balance, { Currency } from "@/components/balance/balance"

export const Account = ({
  account,
  currency,
  depth = 0,
  layer,
  transactionType,
}: {
  account: AccountSetSubAccount
  currency: Currency
  depth?: number
  layer: Layers
  transactionType: TransactionType
}) => {
  return (
    <TableRow key={account.id}>
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
          amount={account.amounts[currency].closingBalance[layer][transactionType]}
        />
      </TableCell>
    </TableRow>
  )
}
