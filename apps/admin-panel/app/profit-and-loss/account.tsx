"use client"
import { TableCell, TableRow } from "@lana/web/ui/table"

import { useRouter } from "next/navigation"

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
  const router = useRouter()

  let accountEnd: number | undefined

  if (account.balanceRange.__typename === "UsdLedgerAccountBalanceRange") {
    accountEnd = account.balanceRange.usdEnd[layer].net
  } else if (account.balanceRange.__typename === "BtcLedgerAccountBalanceRange") {
    accountEnd = account.balanceRange.btcEnd[layer].net
  }

  const handleRowClick = () => {
    router.push(`/ledger-accounts/${account.code || account.id}`)
  }

  return (
    <TableRow
      data-testid={`account-${account.id}`}
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
        <Balance align="end" currency={currency} amount={accountEnd as CurrencyType} />
      </TableCell>
    </TableRow>
  )
}
