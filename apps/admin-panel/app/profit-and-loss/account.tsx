"use client"
import { TableCell, TableRow } from "@lana/web/ui/table"

import { useRouter } from "next/navigation"

import Balance, { Currency } from "@/components/balance/balance"
import { ProfitAndLossStatementQuery } from "@/lib/graphql/generated"
import { ReportLayer } from "@/components/report-filters/selectors"

type AccountType = NonNullable<
  ProfitAndLossStatementQuery["profitAndLossStatement"]
>["categories"][0]["children"][number]

type BalanceRange = AccountType["balanceRange"]

function getBalanceNet(
  balanceRange: BalanceRange,
  currency: Currency,
  layer: ReportLayer,
): number {
  switch (currency) {
    case "usd":
      return balanceRange.usd.usdDiff[layer].net
    case "btc":
      return balanceRange.btc.btcDiff[layer].net
  }
}

interface AccountProps {
  account: AccountType
  currency: Currency
  depth?: number
  layer: ReportLayer
}

export const Account = ({ account, currency, depth = 0, layer }: AccountProps) => {
  const router = useRouter()

  const accountPeriod = getBalanceNet(account.balanceRange, currency, layer)

  const handleRowClick = () => {
    router.push(`/ledger-accounts/${account.code || account.ledgerAccountId}`)
  }

  return (
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
  )
}
