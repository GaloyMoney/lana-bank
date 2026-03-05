"use client"
import { TableCell, TableRow } from "@lana/web/ui/table"

import { useRouter } from "next/navigation"

import Balance, { Currency } from "@/components/balance/balance"
import { ProfitAndLossStatementQuery } from "@/lib/graphql/generated"

type BalanceRangeType = NonNullable<
  ProfitAndLossStatementQuery["profitAndLossStatement"]
>["categories"][0]["children"][number]

export interface ProfitAndLossAccountNode {
  profitAndLossAccountId: string
  ledgerAccountId: string
  code?: string | null
  name: string
  balanceRange: BalanceRangeType["balanceRange"]
  children?: ProfitAndLossAccountNode[]
}

interface AccountProps {
  account: ProfitAndLossAccountNode
  currency: Currency
  depth?: number
  layer: PnlLayers
}

export const Account = ({ account, currency, depth = 0, layer }: AccountProps) => {
  const router = useRouter()

  let accountPeriod: number | undefined

  if (account.balanceRange.__typename === "UsdLedgerAccountBalanceRange") {
    accountPeriod = account.balanceRange.usdDiff[layer].net
  } else if (account.balanceRange.__typename === "BtcLedgerAccountBalanceRange") {
    accountPeriod = account.balanceRange.btcDiff[layer].net
  }

  const handleRowClick = () => {
    router.push(`/ledger-accounts/${account.code || account.ledgerAccountId}`)
  }

  return (
    <>
      <TableRow
        data-testid={`account-${account.profitAndLossAccountId}`}
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
          <Balance align="end" currency={currency} amount={accountPeriod as CurrencyType} />
        </TableCell>
      </TableRow>
      {account.children?.map((child) => (
        <Account
          key={child.profitAndLossAccountId}
          account={child}
          currency={currency}
          depth={depth + 1}
          layer={layer}
        />
      ))}
    </>
  )
}
