"use client"
import { TableCell, TableRow } from "@lana/web/ui/table"

import { useRouter } from "next/navigation"

import Balance, { Currency } from "@/components/balance/balance"
import { BalanceSheetQuery } from "@/lib/graphql/generated"

type BalanceSheetBalance = NonNullable<
  BalanceSheetQuery["balanceSheet"]
>["categories"][number]["balance"]

export interface BalanceSheetAccountNode {
  balanceSheetAccountId: string
  ledgerAccountId: string
  code?: string | null
  name: string
  balance: BalanceSheetBalance
  children?: BalanceSheetAccountNode[]
}

interface AccountProps {
  account: BalanceSheetAccountNode
  currency: Currency
  depth?: number
  layer: BalanceSheetLayers
}

export const Account = ({ account, currency, depth = 0, layer }: AccountProps) => {
  const router = useRouter()

  const balance = account.balance[currency][layer].net

  const handleRowClick = () => {
    router.push(`/ledger-accounts/${account.code || account.ledgerAccountId}`)
  }

  return (
    <>
      <TableRow
        key={account.balanceSheetAccountId}
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
      {account.children?.map((child) => (
        <Account
          key={child.balanceSheetAccountId}
          account={child}
          currency={currency}
          depth={depth + 1}
          layer={layer}
        />
      ))}
    </>
  )
}
